mod choice;
mod confirm;
pub(crate) mod message;

use choice::ChoiceView;
use confirm::ConfirmView;
use message::{MessageLifecycle, MessageView};

use super::data::Data;
use super::dialogue_ctx::ViewActor;
use super::engine::Engine;
use super::line_state::{ChoiceState, ConfirmState, MessageState};

use dialogue::{Choice, ChoiceKey, Confirm, LineType, Message, Nodes, Text, TypingSpeedFactor};

use std::borrow::Cow;

#[derive(Debug, Default, PartialEq, Clone)]
pub enum View<'a> {
    #[default]
    None,
    Terminated(u8),
    Message(MessageView<'a>),
    Confirm(ConfirmView<'a>),
    Choice(ChoiceView<'a>),
}

impl<'a> View<'a> {
    pub fn into_owned(self) -> View<'static> {
        match self {
            View::None => View::None,
            View::Terminated(code) => View::Terminated(code),
            View::Message(mv) => View::Message(mv.into_owned()),
            View::Confirm(cv) => View::Confirm(cv.into_owned()),
            View::Choice(cv) => View::Choice(cv.into_owned()),
        }
    }
}

impl View<'_> {
    pub fn new<'a>(engine: &'a Engine, data: &'a Data, nodes: &'a Nodes) -> View<'a> {
        let state_machine = &data.state_machine;
        if state_machine.call_stack.is_empty() {
            View::Terminated(data.exit_code.unwrap_or_default())
        } else {
            let line_type = data
                .state_machine
                .current_line_type(nodes)
                .expect("Current line type should exist when call stack is not empty");

            match line_type {
                LineType::Message(message) => {
                    View::Message(Self::message_view(engine, data, message, None))
                }
                LineType::Confirm(confirm) => {
                    View::Confirm(Self::confirm_view(engine, data, confirm))
                }
                LineType::Choice(choice) => View::Choice(Self::choice_view(engine, data, choice)),
                _ => todo!("Unimplemented line type"),
            }
        }
    }

    pub fn update<'a>(&mut self, engine: &'a Engine, data: &'a mut Data, nodes: &'a Nodes) -> bool {
        let view = Self::new(engine, data, nodes);

        if *self != view {
            *self = view.into_owned();
            true
        } else {
            false
        }
    }

    fn message_view<'a>(
        engine: &'a Engine,
        data: &'a Data,
        message: &'a Message,
        message_state: Option<&'a MessageState>,
    ) -> MessageView<'a> {
        let config = engine.config();
        let state_machine = &data.state_machine;

        let actor = data
            .dialogue_ctx
            .actor(&message.owner)
            .expect("actor for the message owner should exist");

        let view_actor = actor.view_actor(&config.language);

        let message_state = message_state.unwrap_or_else(|| data.visiting_state_or_panic::<MessageState>());

        let text = &message_state
            .texts
            .get(&config.language)
            .expect("text for the specified language should exist");

        let line_speed_factor = message
            .options
            .as_ref()
            .and_then(|opts| opts.speed)
            .unwrap_or_default();

        let visible_chars_count = Self::visible_chars_count(
            line_speed_factor,
            state_machine.fast_forward.as_ref(),
            message_state,
            &config.typing,
            text,
            &config.language,
        );

        let lifecycle = Self::lifecycle(text, visible_chars_count, message_state);

        MessageView::new(actor, view_actor, text, &message_state.texts, lifecycle)
    }

    fn confirm_view<'a>(
        engine: &'a Engine,
        data: &'a Data,
        confirm: &'a Confirm,
    ) -> ConfirmView<'a> {
        let config = engine.config();
        let cs = data.visiting_state_or_panic::<ConfirmState>();

        let responses = cs.response_texts.as_ref().map(|response_texts| {
            let yes = response_texts
                .yes
                .get(&config.language)
                .expect("yes response text for the specified language should exist");
            let no = response_texts
                .no
                .get(&config.language)
                .expect("no response text for the specified language should exist");
            (yes, no)
        });

        let message_view =
            Self::message_view(engine, data, &confirm.message, Some(&cs.message_state));

        ConfirmView::new(
            cs.confirmed.is_some(),
            responses,
            cs.response_texts.as_ref(),
            message_view,
        )
    }

    fn choice_view<'a>(engine: &'a Engine, data: &'a Data, choice: &'a Choice) -> ChoiceView<'a> {
        let cs = data.visiting_state_or_panic::<ChoiceState>();

        let choices = cs
            .texts
            .iter()
            .map(|(key, texts)| {
                let text = texts
                    .get(&engine.config().language)
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
        let selected = cs.selected.as_ref().map(|s| Cow::Borrowed(&s.choice_key));
        let started_at = Cow::Borrowed(&cs.visited_at);

        let message = choice.options.as_ref().and_then(|options| {
            options
                .message
                .as_ref()
                .map(|message| Self::message_view(engine, data, message, cs.message_state.as_ref()))
        });

        ChoiceView::new(choices, default, timeout, started_at, selected, message)
    }

    fn lifecycle(text: &Text, visible_chars_count: usize, message_state: &MessageState) -> MessageLifecycle {
        match text.char_indices().nth(visible_chars_count) {
            Some(_) => MessageLifecycle::Typing(visible_chars_count),
            None if !message_state.is_completed() => MessageLifecycle::Finished,
            None if message_state.is_skipped() => MessageLifecycle::Completed(message_state.skipped_at.unwrap()),
            None => MessageLifecycle::Completed(message_state.completed_at.unwrap()),
        }
    }

    fn visible_chars_count(
        line_speed_factor: TypingSpeedFactor,
        fast_forward: Option<&std::time::Instant>,
        message_state: &MessageState,
        typing: &super::engine::config::TypingConfig,
        text: &Text,
        language_tag: &language_tags::LanguageTag,
    ) -> usize {
        use std::ops::{Add, AddAssign, Mul};

        if message_state.is_skipped() || message_state.is_completed() {
            return usize::MAX;
        }

        let mut total_fast_forward = message_state.total_fast_forward;

        if let Some(start) = fast_forward {
            total_fast_forward.add_assign(start.elapsed());
        }

        let effective_elapsed = message_state
            .visited_at
            .elapsed()
            .add(total_fast_forward.mul_f32(*typing.fast_forward_factor))
            .saturating_sub(
                (!message_state.initial_fast_forward)
                    .then(|| typing.start_delay)
                    .unwrap_or_default(),
            );

        effective_elapsed
            .mul_f32(
                typing
                    .effective_speed(text, language_tag)
                    .mul(*line_speed_factor),
            )
            .as_secs() as usize
    }
}

pub enum Selectable<'a> {
    Confirm(Option<(&'a str, &'a str)>),
    Choice(Vec<(&'a ChoiceKey, &'a str)>),
}

impl View<'static> {
    pub fn actor(&self) -> Option<&ViewActor> {
        match &self {
            View::Message(message_view) => Some(message_view.view_actor()),
            View::Confirm(confirm_view) => Some(confirm_view.message_view.view_actor()),
            View::Choice(choice_view) => choice_view
                .message_view()
                .as_ref()
                .and_then(|message_view| Some(message_view.view_actor())),
            _ => None,
        }
    }

    pub fn message(&self) -> Option<&str> {
        match &self {
            View::Message(message_view) => Some(message_view.visible_str()),
            View::Confirm(confirm_view) => Some(confirm_view.visible_str()),
            View::Choice(choice_view) => choice_view
                .message_view()
                .as_ref()
                .map(|message_view| message_view.visible_str()),
            _ => None,
        }
    }

    pub fn selectable(&self) -> Option<Selectable<'_>> {
        match &self {
            View::Choice(_) => self.choices().map(Selectable::Choice),
            View::Confirm(confirm_view) => confirm_view
                .is_available()
                .then_some(Selectable::Confirm(confirm_view.responses())),
            _ => None,
        }
    }

    pub fn responses(&self) -> Option<(&str, &str)> {
        match &self {
            View::Confirm(confirm_view) => confirm_view
                .responses
                .as_ref()
                .map(|(yes, no)| (yes.as_ref().as_str(), no.as_ref().as_str())),
            _ => None,
        }
    }

    pub fn choices(&self) -> Option<Vec<(&ChoiceKey, &str)>> {
        match &self {
            View::Choice(choice_view) => choice_view.choices_available().map(|choices| {
                choices
                    .into_iter()
                    .map(|(k, v)| (k.as_ref(), v.as_ref().as_str()))
                    .collect()
            }),
            _ => None,
        }
    }

    pub fn has_message(&self) -> bool {
        match &self {
            View::Message(_) | View::Confirm(_) => true,
            View::Choice(choice_view) => choice_view.has_message(),
            _ => false,
        }
    }

    pub fn has_message_finished(&self) -> bool {
        match &self {
            View::Message(message_view) => message_view.is_finished(),
            View::Confirm(confirm_view) => confirm_view.is_finished(),
            View::Choice(choice_view) => choice_view
                .message_view()
                .as_ref()
                .map_or(false, |message_view| message_view.is_finished()),
            _ => false,
        }
    }

    pub fn has_available_selectable(&self) -> bool {
        self.has_available_choice() || self.has_available_confirm()
    }

    pub fn has_available_confirm(&self) -> bool {
        matches!(self, View::Confirm(confirm) if confirm.is_available())
    }

    pub fn has_available_choice(&self) -> bool {
        matches!(self, View::Choice(choice) if choice.is_available())
    }
}

// These methods may not need.
impl View<'static> {
    pub fn is_message(&self) -> bool {
        matches!(self, View::Message(_))
    }

    pub fn is_choice(&self) -> bool {
        matches!(self, View::Choice(_))
    }

    pub fn is_terminated(&self) -> bool {
        matches!(self, View::Terminated(_))
    }

    pub fn as_message(&self) -> Option<&MessageView> {
        match &self {
            View::Message(message_view) => Some(message_view),
            View::Confirm(confirm_view) => Some(&confirm_view.message_view),
            View::Choice(choice_view) => choice_view.message_view().as_ref(),
            _ => None,
        }
    }

    pub fn as_choice(&self) -> Option<&ChoiceView> {
        match &self {
            View::Choice(choice_view) => Some(choice_view),
            _ => None,
        }
    }
}
