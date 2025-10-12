mod choice;
pub(crate) mod message;

use choice::ChoiceView;
use message::{MessageLifecycle, MessageView};

use super::data::Data;
use super::dialogue_ctx::Actor;
use super::engine::Engine;
use super::state_machine::StateMachine;
use super::visiting_state::MessageVisitingState;

use dialogue::{Choice, ChoiceKey, LineType, Message, Nodes, Text};

use std::borrow::Cow;

#[derive(Debug, Default, PartialEq)]
pub enum View<'a> {
    #[default]
    Uninitialized,
    Terminated,
    Message(MessageView<'a>),
    Choice(ChoiceView<'a>),
}

impl<'a> View<'a> {
    pub fn into_owned(self) -> View<'static> {
        match self {
            View::Uninitialized => View::Uninitialized,
            View::Terminated => View::Terminated,
            View::Message(mv) => View::Message(mv.into_owned()),
            View::Choice(cv) => View::Choice(cv.into_owned()),
        }
    }
}

impl View<'_> {
    pub fn update<'a>(&mut self, engine: &Engine, data: &'a mut Data, nodes: &'a Nodes) -> bool {
        let state_machine = &data.state_machine;
        let current_line_type = state_machine.current_line_type(nodes);
        let view = if state_machine.is_terminated() {
            View::Terminated
        } else {
            match current_line_type {
                LineType::Message(message) => {
                    View::Message(Self::message_view(engine, data, nodes, message, None))
                }
                LineType::Choice(choice) => {
                    View::Choice(Self::choice_view(engine, data, nodes, choice))
                }
                _ => todo!("Unimplemented line type"),
            }
        };

        if *self != view {
            *self = view.into_owned();
            true
        } else {
            false
        }
    }

    fn message_view<'a>(
        engine: &Engine,
        data: &'a Data,
        nodes: &Nodes,
        message: &Message,
        message_visiting_state: Option<&'a MessageVisitingState>,
    ) -> MessageView<'a> {
        let state_machine = &data.state_machine;

        let actor = data
            .dialogue_ctx
            .actor(&message.owner)
            .expect("actor for the message owner should exist");

        let mvs = message_visiting_state.unwrap_or_else(|| data.message_visiting_state());

        let text = mvs
            .evaluated_texts
            .get(engine.config().language())
            .expect("text for the specified language should exist");

        let visible_chars_count =
            Self::visible_chars_count(nodes, state_machine, mvs, engine.config().typing());

        let lifecycle = match text.char_indices().nth(visible_chars_count) {
            Some((idx, _)) => MessageLifecycle::Typing(idx),
            None if !mvs.is_completed() => MessageLifecycle::Finished,
            None if mvs.is_skipped() => MessageLifecycle::Completed(mvs.skipped_at.unwrap()),
            None => MessageLifecycle::Completed(mvs.completed_at.unwrap()),
        };

        MessageView::new(actor, text, lifecycle)
    }

    fn choice_view<'a>(
        engine: &Engine,
        data: &'a Data,
        nodes: &Nodes,
        choice: &'a Choice,
    ) -> ChoiceView<'a> {
        let cvs = data.choice_visiting_state();

        let choices = cvs
            .evaluated_texts
            .iter()
            .map(|(key, texts)| {
                let text = texts
                    .get(engine.config().language())
                    .expect("text for the specified language should exist");
                (Cow::Borrowed(key), Cow::Borrowed(text))
            })
            .collect::<Vec<_>>();
        let default = choice
            .options
            .as_ref()
            .and_then(|options| options.default.as_ref().map(|d| Cow::Borrowed(d)));
        let timeout = choice.options.as_ref().and_then(|options| {
            options
                .timeout
                .as_ref()
                .map(|timeout| Cow::Borrowed(timeout))
        });
        let selected = cvs.selected.as_ref().map(|s| Cow::Borrowed(&s.choice_key));
        let started_at = Cow::Borrowed(&cvs.started_at);

        let message = choice.options.as_ref().and_then(|options| {
            options.message.as_ref().map(|message| {
                Self::message_view(
                    engine,
                    data,
                    nodes,
                    message,
                    cvs.message_visiting_state.as_ref(),
                )
            })
        });

        ChoiceView::new(choices, default, timeout, started_at, selected, message)
    }

    fn visible_chars_count(
        nodes: &Nodes,
        state_machine: &StateMachine,
        mvs: &MessageVisitingState,
        typing: &super::engine::config::Typing,
    ) -> usize {
        use std::ops::{Add, AddAssign, Mul};

        if mvs.is_skipped() || mvs.is_completed() {
            return usize::MAX;
        }

        let mut total_fast_forward = mvs.total_fast_forward;

        if let Some(start) = state_machine.fast_forward.as_ref() {
            total_fast_forward.add_assign(start.elapsed());
        }

        let effective_elapsed = mvs
            .started_at
            .elapsed()
            .add(total_fast_forward.mul_f32(typing.fast_forward_speed()))
            .saturating_sub(
                (!mvs.initial_fast_forward)
                    .then(|| typing.start_delay().clone())
                    .unwrap_or_default(),
            );

        let line_speed = state_machine
            .current_line_speed(nodes)
            .map(|s| s.clone())
            .unwrap_or_default();

        effective_elapsed
            .mul_f32(typing.speed().mul(*line_speed))
            .as_secs() as usize
    }
}

impl View<'static> {
    pub fn actor(&self) -> Option<&Cow<'_, Actor>> {
        match &self {
            View::Message(message_view) => Some(message_view.actor()),
            View::Choice(choice_view) => choice_view
                .message_view()
                .as_ref()
                .and_then(|message_view| Some(message_view.actor())),
            _ => None,
        }
    }

    pub fn message(&self) -> Option<&str> {
        match &self {
            View::Message(message_view) => Some(message_view.visible_str()),
            View::Choice(choice_view) => choice_view
                .message_view()
                .as_ref()
                .map(|message_view| message_view.visible_str()),
            _ => None,
        }
    }

    pub fn choices(&self) -> Option<Vec<(ChoiceKey, Text)>> {
        match &self {
            View::Choice(choice_view) => choice_view.choices_available().map(|choices| {
                choices
                    .into_iter()
                    .map(|(k, v)| (k.as_ref().clone(), v.as_ref().clone()))
                    .collect()
            }),
            _ => None,
        }
    }

    pub fn has_message(&self) -> bool {
        match &self {
            View::Message(_) => true,
            View::Choice(choice_view) => choice_view.has_message(),
            _ => false,
        }
    }

    pub fn has_message_finished(&self) -> bool {
        match &self {
            View::Message(message_view) => message_view.is_finished(),
            View::Choice(choice_view) => choice_view
                .message_view()
                .as_ref()
                .map_or(false, |message_view| message_view.is_finished()),
            _ => false,
        }
    }

    pub fn has_available_choice(&self) -> bool {
        match &self {
            View::Choice(choice_view) => choice_view.is_available(),
            _ => false,
        }
    }

    pub fn is_message(&self) -> bool {
        matches!(self, View::Message(_))
    }

    pub fn is_choice(&self) -> bool {
        matches!(self, View::Choice(_))
    }

    pub fn is_terminated(&self) -> bool {
        matches!(self, View::Terminated)
    }
}
