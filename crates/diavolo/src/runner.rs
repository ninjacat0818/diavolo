mod action_handler_impl;

use super::boa_ctx::BoaCtx;
use super::store::Store;
use super::view::View;

use dialogue::{ChoiceKey, Dialogue, LineType, NodeKey};

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

    pub fn view(&self) -> &View<'static> {
        &self.view
    }

    pub fn update_view(&mut self) -> bool {
        self.view.update(
            &self.store.engine,
            &mut self.store.data.lock().unwrap(),
            &self.dialogue.nodes,
        )
    }

    pub fn dispatch(&mut self, action: Action) {
        let result = match action {
            Action::CompleteMessage => self.handle_complete_message(),
            Action::NextLine => self.handle_next_line(),
            Action::ToggleFastForward => self.handle_toggle_fast_forward(),
            Action::Skip => self.handle_skip(),
            Action::Select(ref choice_key) => self.handle_select(choice_key),
        };

        self.update_view();

        if let Err(e) = &result {
            tracing::warn!("Error handling action {:?}: {}", action, e);
        }
    }
}

impl Runner<'_, '_> {
    fn init(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        let mut data = self.store.data.lock().unwrap();

        if data.state_machine.is_initialized() {
            return Err("StateMachine is already initialized".into());
        }

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

        drop(data);

        let _ = self.advance()?;
        self.update_view();

        Ok(self)
    }

    fn advance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut next = Some(());

        while let Some(_) = next.take() {
            let is_initialized = {
                let data = self.store.data.lock().unwrap();
                data.state_machine.is_initialized()
            };

            if is_initialized {
                let _ = self.commit_fast_forward();

                let state_machine = &mut self.store.data.lock().unwrap().state_machine;

                if state_machine.is_last_line(&self.dialogue.nodes) {
                    tracing::debug!("Already at the last line, cannot advance further");
                    state_machine.terminate();
                    return Ok(());
                }
                state_machine.advance();
            } else {
                let mut data = self.store.data.lock().unwrap();

                if self
                    .dialogue
                    .nodes
                    .get(&NodeKey::main())
                    .expect("main node should exist")
                    .is_empty()
                {
                    tracing::debug!("Main node is empty, terminating dialogue immediately");
                    data.state_machine.terminate();
                    return Ok(());
                }

                data.state_machine.initialize();
                data.visiting_states.init_node(
                    &NodeKey::main(),
                    self.dialogue.nodes.get(&NodeKey::main()).unwrap(),
                );
            }

            let line_if_result = {
                let data = self.store.data.lock().unwrap();
                data.state_machine
                    .current_line_if(&self.dialogue.nodes)
                    .map(|line_if| {
                        drop(data);
                        self.boa_ctx.eval_if(&line_if)
                    })
            }
            .transpose()?
            .unwrap_or(true);

            if line_if_result {
                let evaluated_texts = {
                    let data = self.store.data.lock().unwrap();
                    let line_type = data.state_machine.current_line_type(&self.dialogue.nodes);
                    match line_type {
                        LineType::Message(message) => {
                            drop(data);
                            Some(self.boa_ctx.eval_texts(&message.texts)?)
                        }
                        LineType::Choice(choice) => choice
                            .message()
                            .map(|message| {
                                drop(data);
                                self.boa_ctx.eval_texts(&message.texts)
                            })
                            .transpose()?,
                        _ => None,
                    }
                };

                let evaluated_choice_texts = {
                    let data = self.store.data.lock().unwrap();
                    let line_type = data.state_machine.current_line_type(&self.dialogue.nodes);
                    match line_type {
                        LineType::Choice(choice) => {
                            drop(data);
                            Some(self.boa_ctx.eval_choice_texts(&choice.texts)?)
                        }
                        _ => None,
                    }
                };

                let mut data = self.store.data.lock().unwrap();
                data.visit_line(evaluated_texts, evaluated_choice_texts)?;
            } else {
                tracing::debug!("Line if condition evaluated to false, skipping line");
                next.replace(());
            }
        }

        Ok(())
    }

    fn commit_fast_forward(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.store.data.lock().unwrap();
        let current_line_type = data.state_machine.current_line_type(&self.dialogue.nodes);

        let start = data
            .state_machine
            .fast_forward
            .ok_or("Not in fast forward state")?;

        match current_line_type {
            LineType::Message(_) => {
                data.message_visiting_state_mut()
                    .commit_fast_forward(start.elapsed());
                Ok(())
            }
            LineType::Choice(_) => data
                .choice_visiting_state_mut()
                .try_commit_fast_forward(start.elapsed()),
            _ => Err("Cannot commit fast forward on non-message line".into()),
        }?;

        data.state_machine.enter_fast_forward();
        Ok(())
    }
}

#[derive(Debug)]
pub enum Action {
    CompleteMessage,
    NextLine,
    ToggleFastForward,
    Skip,
    Select(ChoiceKey),
    // Confirm(bool),
}

#[cfg(test)]
mod tests {
    use boa_engine::Source;

    use super::*;
    use crate::{Data, Dialogue, DialogueCtx, Engine, Store};

    #[test]
    fn boa_test() {
        let dialogue: Dialogue = r#"
actor:
  num: 1
nodes:
  main:
  - id: line1
    choice:
      foo:
        en: Foo
  - id: line2
    if: lines.line1.selected === "foo"
    message:
      en: line2 text
    owner: 0
"#
        .parse()
        .unwrap();

        let actors = serde_json::json!([{ "name": "Actor1" }]);
        let engine = &Engine::default();
        let data = Data::with_ctx(DialogueCtx::builder().actors(actors).build());
        let store = &mut Store::new(engine, data);
        let mut runner = Runner::instantiate(store, &dialogue).unwrap();

        let js_assert = r#"
            function assert(condition, message) { if (!condition) { throw new Error(message); } }
            function assert_eq(lhs, rhs, message) { if (lhs !== rhs) { throw new Error(message); } }
        "#;
        runner.boa_ctx.eval(Source::from_bytes(js_assert)).unwrap();

        let source = r#"
            assert_eq(lines[0].id, "line1", "Line ID should be line1");
            assert_eq(lines[0].visited, true, "Line visited should be true");
            assert_eq(lines[0].visited_count, 1, "Line visited count should be 1");
            assert_eq(lines[0].selected, undefined, "Line selected should be undefined");
            assert_eq(lines.line1.id, lines[0].id, "Line access by ID should work");

            assert_eq(lines[1].id, "line2", "Line ID should be line2");
            assert_eq(lines[1].visited, false, "Line visited should be false");
            assert_eq(lines[1].visited_count, 0, "Line visited count should be 0");
        "#;

        let _result = runner
            .boa_ctx
            .eval(Source::from_bytes(source))
            .expect("Boa script should run without errors");

        runner.dispatch(Action::Select(ChoiceKey::new("foo")));
        runner.dispatch(Action::NextLine);

        let source = r#"
            assert_eq(lines.line1.selected, "foo", "Line ID should be line1");
            assert_eq(lines.line2.visited, true, "Line visited should be true");
            assert_eq(lines.line2.visited_count, 1, "Line visited count should be 1");
        "#;

        let _result = runner
            .boa_ctx
            .eval(Source::from_bytes(source))
            .expect("Boa script should run without errors");
    }
}
