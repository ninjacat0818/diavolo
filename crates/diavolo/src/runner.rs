mod action_handler_impl;
mod evaluated_line;

use super::boa_ctx::BoaCtx;
use super::line_state::{ChoiceState, ConfirmState, MessageState};
use super::store::Store;
use super::view::View;
use dialogue::{ChoiceKey, ConfirmResponse, Dialogue, LineType};
pub(crate) use evaluated_line::EvaluatedLine;

use std::ops::ControlFlow;

pub struct Runner<'e, 'd> {
    store: &'e mut Store<'e>,
    dialogue: &'d Dialogue,
    boa_ctx: BoaCtx,
    view: View<'static>,
}

impl<'engine, 'dialogue> Runner<'engine, 'dialogue> {
    pub fn instantiate(
        store: &'engine mut Store<'engine>,
        dialogue: &'dialogue Dialogue,
    ) -> Result<Runner<'engine, 'dialogue>, Box<dyn std::error::Error>> {
        Self {
            store,
            dialogue,
            boa_ctx: BoaCtx::default(),
            view: View::default(),
        }
        .init()
    }

    pub fn is_terminated(&self) -> bool {
        matches!(
            View::new(
                &self.store.engine,
                &mut self.store.data.lock().unwrap(),
                &self.dialogue.nodes,
            ),
            View::Terminated(_)
        )
    }

    pub fn view(&self) -> &View<'static> {
        &self.view
    }

    pub fn update_view(&mut self) -> Option<&View<'static>> {
        let mut data = self.store.data.lock().unwrap();

        let updated = self
            .view
            .update(&self.store.engine, &mut data, &self.dialogue.nodes);

        if updated && self.view.has_message_finished() {
            tracing::debug!("Completing message automatically as it has finished");
            tracing::trace!("Current view: {:?}", self.view);
            data.complete_message_or_panic(&self.view);
            self.view
                .update(&self.store.engine, &mut data, &self.dialogue.nodes);
        }

        updated.then_some(&self.view)
    }

    pub fn dispatch(&mut self, action: Action) -> bool {
        let result = match action {
            Action::Advance => self.handle_advance(),
            Action::ToggleFastForward => self.handle_toggle_fast_forward(),
            Action::Skip => self.handle_skip(),
            Action::Confirm(approved) => self.handle_confirm(approved),
            Action::Select(ref choice_key) => self.handle_select(choice_key),
        };

        match result {
            Ok(_) => true,
            Err(e) => {
                tracing::warn!("Error handling action {:?}: {}", action, e);
                false
            }
        }
    }
}

impl Runner<'_, '_> {
    fn init(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        let mut data = self.store.data.lock().unwrap();

        if self.dialogue.actor_num() != data.dialogue_ctx.actors_count() {
            return Err(format!(
                "Dialogue expects {} actors, but DialogueCtx has {} actors",
                self.dialogue.actor_num(),
                data.dialogue_ctx.actors_count()
            )
            .into());
        }

        self.boa_ctx.define_properties(self.store.data.clone())?;

        if let Some(args) = data.dialogue_ctx.parsed_args(&self.dialogue.args)? {
            args.register_in_boa_context(&mut self.boa_ctx)?;
        };

        data.call(&self.dialogue.nodes);
        drop(data);
        self.advance()?;

        Ok(self)
    }

    fn advance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut skipped: Option<()> = None;

        loop {
            let mut data = self.store.data.lock().unwrap();
            let state_machine = &data.state_machine;
            if state_machine.call_stack.is_empty() {
                tracing::debug!("Call stack is empty, dialogue execution finished");
                break Ok(());
            }

            if let None = skipped.take() {
                if let Err(e) = Self::try_commit_fast_forward(&mut data, &self.dialogue.nodes, true)
                {
                    tracing::debug!("Failed to commit fast forward: {}", e);
                };
            }

            if data.state_machine.is_last_line(&self.dialogue.nodes) {
                tracing::debug!("Cannot advance further, returning to caller");
                data.ret();
                continue;
            }
            data.state_machine.advance();
            drop(data);

            match self.evaluate()? {
                ControlFlow::Continue(ContinueReason::Skip) => skipped = Some(()),
                ControlFlow::Continue(ContinueReason::ControlLine) => (),
                ControlFlow::Break(_) => break Ok(()),
            }
        }
    }

    fn evaluate(&mut self) -> Result<ControlFlow<(), ContinueReason>, Box<dyn std::error::Error>> {
        let mut loop_counter = 0;

        loop {
            loop_counter += 1;
            if loop_counter > 1000 {
                return Err("Infinite loop detected in dialogue evaluation".into());
            }

            let line_if_result = {
                (self.store.data.lock().unwrap())
                    .state_machine
                    .current_line_if(&self.dialogue.nodes)
            }
            .map(|line_if| self.boa_ctx.eval_if(&line_if))
            .transpose()?
            .unwrap_or(true);

            if !line_if_result {
                tracing::debug!("Line if condition evaluated to false, skipping line");
                break Ok(ControlFlow::Continue(ContinueReason::Skip));
            }

            let data = self.store.data.lock().unwrap();
            let state_machine = &data.state_machine;
            let line_type = state_machine
                .current_line_type(&self.dialogue.nodes)
                .expect("Current line type should exist");
            drop(data);

            let ctx = &mut self.boa_ctx;

            let evaluated_line = {
                match line_type {
                    LineType::Message(message) => {
                        EvaluatedLine::Message(ctx.eval_texts(&message.texts)?)
                    }
                    LineType::Confirm(confirm) => {
                        let texts = ctx.eval_texts(&confirm.message.texts)?;
                        let response_texts = confirm
                            .options
                            .as_ref()
                            .and_then(|options| options.response.as_ref())
                            .map(|response_texts| {
                                Ok::<_, boa_engine::JsError>(ConfirmResponse {
                                    yes: ctx.eval_texts(&response_texts.yes)?,
                                    no: ctx.eval_texts(&response_texts.no)?,
                                })
                            })
                            .transpose()?;
                        EvaluatedLine::Confirm(texts, response_texts)
                    }
                    LineType::Choice(choice) => {
                        let choice_texts = ctx.eval_choice_texts(&choice.texts)?;
                        let texts = choice
                            .message()
                            .map(|message| ctx.eval_texts(&message.texts))
                            .transpose()?;
                        EvaluatedLine::Choice(choice_texts, texts)
                    }
                    LineType::Eval(eval) => {
                        tracing::debug!("Evaluating Eval line: {}", eval.source);
                        EvaluatedLine::Eval(ctx.eval(boa_engine::Source::from_bytes(&eval.source))?)
                    }
                    LineType::Goto(goto) => {
                        let text = ctx.eval_text(&goto.pre_evaluation_value.as_str().into())?;
                        EvaluatedLine::Goto(text.to_string())
                    }
                    LineType::Call(call) => {
                        EvaluatedLine::Call(ctx.eval_text(&call.pre_evaluation_node_key)?.into())
                    }
                    LineType::Return(r#return) => {
                        EvaluatedLine::Return(ctx.eval_str(&r#return.pre_evaluation_value)?)
                    }
                    LineType::Exit(exit) => {
                        use dialogue::ExitValue;
                        let code = match &exit.value {
                            ExitValue::PreEvaluation(source) => ctx
                                .eval(boa_engine::Source::from_bytes(source))?
                                .to_uint8(ctx)?,
                            ExitValue::ExitCode(code) => *code,
                        };
                        self.store.data.lock().unwrap().exit(code);
                        break Ok(ControlFlow::Continue(ContinueReason::ControlLine));
                    }
                }
            };

            let mut data = self.store.data.lock().unwrap();
            data.visit_line(evaluated_line);

            match &line_type {
                LineType::Goto(_) => data.goto(&self.dialogue.nodes),
                LineType::Call(_) => data.call(&self.dialogue.nodes),
                LineType::Return(_) => data.ret(),
                _ => (),
            }

            match line_type {
                LineType::Goto(_) => continue,
                LineType::Eval(_) | LineType::Call(_) | LineType::Return(_) => {
                    break Ok(ControlFlow::Continue(ContinueReason::ControlLine));
                }
                _ => break Ok(ControlFlow::Break(())),
            }
        }
    }

    fn try_commit_fast_forward(
        data: &mut super::data::Data,
        nodes: &dialogue::Nodes,
        re_enter: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let fast_forward = &mut data.state_machine.fast_forward;

        let start = (fast_forward.is_some() && re_enter)
            .then(|| fast_forward.replace(std::time::Instant::now()))
            .unwrap_or(fast_forward.take())
            .ok_or("Not in fast forward state")?;

        let msg = "No current line to commit fast forward. Probably the first line has not been visited yet";
        let current_line_type = data.state_machine.current_line_type(nodes).ok_or(msg)?;

        match current_line_type {
            LineType::Message(_) => data
                .visiting_state_mut::<MessageState>()
                .map(|message_state| message_state.commit_fast_forward(start.elapsed()))
                .ok_or("No message visiting state found"),
            LineType::Confirm(_) => data
                .visiting_state_mut::<ConfirmState>()
                .map(|cs| cs.commit_fast_forward(start.elapsed()))
                .ok_or("No confirm visiting state found"),
            LineType::Choice(_) => data
                .visiting_state_mut::<ChoiceState>()
                .map(|cs| cs.try_commit_fast_forward(start.elapsed()))
                .transpose()?
                .ok_or("No choice visiting state found"),
            _ => Err("Cannot commit fast forward on non-message line".into()),
        }?;

        Ok(())
    }
}

#[derive(Debug)]
enum ContinueReason {
    Skip,
    ControlLine,
}

#[derive(Debug)]
pub enum Action {
    Advance,
    ToggleFastForward,
    Skip,
    Confirm(bool),
    Select(ChoiceKey),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Data, Dialogue, DialogueCtx, Engine, Store};

    #[test]
    fn test_confirm() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init()
            .ok();

        let dialogue: Dialogue = r#"
nodes:
  main:
  - confirm: OK?
"#
        .parse()
        .unwrap();

        let engine = Engine::default();
        let mut store: Store<'_> = Store::new(&engine, Data::default());
        let mut runner = Runner::instantiate(&mut store, &dialogue).unwrap();
        runner.dispatch(Action::Skip);
        runner.dispatch(Action::Confirm(true));
        dbg!(runner.update_view());
        assert!(matches!(runner.view(), View::Terminated(_)));
    }

    #[test]
    fn test_goto() {
        let dialogue: Dialogue = r#"
nodes:
  main:
  - goto: 1
  - goto: line_id_123
  - id: line_id_123
    message: ok
"#
        .parse()
        .unwrap();

        let engine = Engine::default();
        let mut store = Store::new(&engine, Data::default());
        let mut runner = Runner::instantiate(&mut store, &dialogue).unwrap();
        runner.update_view();
        assert!(matches!(runner.view(), View::Message(_)));
    }

    #[test]
    fn test_call() {
        let dialogue: Dialogue = r#"
nodes:
  main:
  - call: foo
  foo:
"#
        .parse()
        .unwrap();

        let engine = Engine::default();
        let mut store = Store::new(&engine, Data::default());
        let mut runner = Runner::instantiate(&mut store, &dialogue).unwrap();
        runner.update_view();
        assert_eq!(runner.view(), &View::Terminated(0));
    }

    #[test]
    fn test_return() {
        let dialogue: Dialogue = r#"
nodes:
  main:
  - call: foo
  - call: bar
  - if: lines[0].returned === undefined && lines[1].returned === 42
    message: ok
  foo:
  - return:
  bar:
  - return: 42
"#
        .parse()
        .unwrap();

        let engine = Engine::default();
        let mut store = Store::new(&engine, Data::default());
        let mut runner = Runner::instantiate(&mut store, &dialogue).unwrap();
        let source = r#"
            assert_eq(lines[0].returned, undefined, "Returned value should be undefined");
            assert_eq(lines[1].returned, 42, "Returned value should be 42");
        "#;
        runner.boa_ctx.eval_for_assert(source);
        runner.update_view();
        assert!(matches!(runner.view(), View::Message(_)));
    }

    #[test]
    fn test_exit() {
        let engine = Engine::default();

        let dialogue1: Dialogue = "nodes:\n  main:\n  - exit: 3".parse().unwrap();
        let store = &mut Store::new(&engine, Data::default());
        let mut runner = Runner::instantiate(store, &dialogue1).unwrap();
        assert_eq!(runner.update_view(), Some(&View::Terminated(3)));

        let dialogue2: Dialogue = "nodes:\n  main:\n  - exit: '1 + 2'".parse().unwrap();
        let store = &mut Store::new(&engine, Data::default());
        let mut runner = Runner::instantiate(store, &dialogue2).unwrap();
        assert_eq!(runner.update_view(), Some(&View::Terminated(3)));

        let dialogue3: Dialogue = "nodes:\n  main:".parse().unwrap();
        let store = &mut Store::new(&engine, Data::default());
        let mut runner = Runner::instantiate(store, &dialogue3).unwrap();
        assert_eq!(runner.update_view(), Some(&View::Terminated(0)));
    }

    #[test]
    fn test_boa() {
        let dialogue: Dialogue = r#"
nodes:
  main:
  - id: line1
    if: self.visited === false
    choice:
      foo: foo
  - id: line2
    if: lines.line1.selected === "foo"
    message: line2 text
"#
        .parse()
        .unwrap();

        let engine = Engine::default();
        let store = &mut Store::new(&engine, Data::default());
        let mut runner = Runner::instantiate(store, &dialogue).unwrap();

        let source = r#"
            assert_eq(lines[0].id, "line1", "Line ID should be line1");
            assert_eq(lines[0].visited, true, "Line visited should be true");
            assert_eq(lines[0].visited_count, 1, "Line visited count should be 1");
            assert_eq(lines[0].visited_count_next, 2, "Line next visited count should be 2");
            assert_eq(lines[0].selected, undefined, "Line selected should be undefined");
            assert_eq(lines.line1.id, lines[0].id, "Line access by ID should work");
            assert_eq(self.id, lines[0].id, "Self line access should work");
            assert_eq(prev, undefined, "Prev line access should be undefined");
            assert_eq(next.id, lines[1].id, "Next line access should work");

            assert_eq(lines[1].id, "line2", "Line ID should be line2");
            assert_eq(lines[1].visited, false, "Line visited should be false");
            assert_eq(lines[1].visited_count, 0, "Line visited count should be 0");
        "#;
        runner.boa_ctx.eval_for_assert(source);

        runner.dispatch(Action::Select(ChoiceKey::new("foo")));
        runner.dispatch(Action::Advance);

        let source = r#"
            assert_eq(lines.line1.selected, "foo", "Line ID should be line1");
            assert_eq(lines.line2.visited, true, "Line visited should be true");
            assert_eq(lines.line2.visited_count, 1, "Line visited count should be 1");
        "#;
        runner.boa_ctx.eval_for_assert(source);
    }
}
