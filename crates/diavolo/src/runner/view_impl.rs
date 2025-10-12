use super::super::state_machine::visiting_states::visiting_state::{
    ChoiceVisitingState, MessageVisitingState,
};
use super::Runner;
use super::view::choice::ChoiceView;
use super::view::message::{MessageLifecycle, MessageView};
use dialogue::{Choice, Message};

use std::borrow::Cow;
use std::ops::{Add, AddAssign, Mul};

impl<'d> Runner<'_, '_> {
    pub(super) fn message_view(self: &Self, message: &'d Message) -> MessageView<'d> {
        self.message_view_internal(message, None)
    }

    fn message_view_internal(
        self: &Self,
        message: &'d Message,
        visiting_state: Option<&MessageVisitingState>,
    ) -> MessageView<'d> {
        let text = message
            .texts
            .get(&self.config().language)
            .expect("Text for the specified language not found");

        let mvs = visiting_state.unwrap_or_else(|| {
            self.state()
                .visiting_state::<MessageVisitingState>()
                .unwrap()
        });

        let visible_chars_count = self.visible_chars_count(mvs);

        let lifecycle = match text.char_indices().nth(visible_chars_count) {
            Some((idx, _)) => MessageLifecycle::Typing(idx),
            None if !mvs.is_completed() => MessageLifecycle::Finished,
            None if mvs.is_skipped() => MessageLifecycle::Completed(mvs.skipped_at().unwrap()),
            None => MessageLifecycle::Completed(mvs.completed_at().unwrap()),
        };

        MessageView::new(Cow::Borrowed(text), lifecycle)
    }

    fn visible_chars_count(&self, mvs: &MessageVisitingState) -> usize {
        if mvs.is_skipped() || mvs.is_completed() {
            return usize::MAX;
        }

        let mut total_fast_forward = mvs.total_fast_forward().clone();

        if let Some(start) = self.state().fast_forward.as_ref() {
            total_fast_forward.add_assign(start.elapsed());
        }

        let effective_elapsed = mvs
            .started_at()
            .elapsed()
            .add(total_fast_forward.mul_f32(self.config().typing.fast_forward_speed))
            .saturating_sub(
                (!mvs.is_initial_fast_forward())
                    .then(|| self.config().typing.start_delay)
                    .unwrap_or_default(),
            );

        let line_speed = self
            .current_line_speed()
            .map(|s| s.clone())
            .unwrap_or_default();

        effective_elapsed
            .mul_f32(self.config().typing.speed.mul(*line_speed))
            .as_secs() as usize
    }

    pub(super) fn choice_view(self: &'d Self, choice: &'d Choice) -> ChoiceView<'d> {
        let cvs = self
            .state()
            .visiting_state::<ChoiceVisitingState>()
            .expect("ChoiceVisitingState not found");

        let choices = choice
            .texts
            .iter()
            .map(|(key, texts)| {
                let text = texts
                    .get(&self.config().language)
                    .expect("Text for the specified language not found");
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
        let selected = cvs.selected().map(|s| Cow::Borrowed(s.choice_key()));
        let started_at = Cow::Borrowed(cvs.started_at());

        let message = choice.options.as_ref().and_then(|options| {
            options.message.as_ref().map(|message| {
                self.message_view_internal(message, cvs.message_visiting_state().as_ref())
            })
        });

        ChoiceView::new(choices, default, timeout, started_at, selected, message)
    }
}
