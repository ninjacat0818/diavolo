use dialogue::{ChoiceKey, Text, Timeout};

use super::message::MessageView;

use std::borrow::Cow;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Clone)]
pub struct ChoiceView<'a> {
    choices: Vec<(Cow<'a, ChoiceKey>, Cow<'a, Text>)>,
    default: Option<Cow<'a, ChoiceKey>>,
    timeout: Option<Cow<'a, Timeout>>,
    started_at: Cow<'a, Instant>,
    selected: Option<Cow<'a, ChoiceKey>>,
    message_view: Option<MessageView<'a>>,
}

impl<'a> ChoiceView<'a> {
    pub fn new(
        choices: Vec<(Cow<'a, ChoiceKey>, Cow<'a, Text>)>,
        default: Option<Cow<'a, ChoiceKey>>,
        timeout: Option<Cow<'a, Timeout>>,
        started_at: Cow<'a, Instant>,
        selected: Option<Cow<'a, ChoiceKey>>,
        message_view: Option<MessageView<'a>>,
    ) -> Self {
        Self {
            choices,
            default,
            timeout,
            started_at,
            selected,
            message_view,
        }
    }

    pub fn into_owned(self) -> ChoiceView<'static> {
        let choices = self
            .choices
            .into_iter()
            .map(|(k, v)| (Cow::Owned(k.into_owned()), Cow::Owned(v.into_owned())))
            .collect();
        let default = self.default.map(|k| Cow::Owned(k.into_owned()));
        let timeout = self.timeout.map(|t| Cow::Owned(t.into_owned()));
        let selected = self.selected.map(|k| Cow::Owned(k.into_owned()));
        let started_at = Cow::Owned(self.started_at.into_owned());
        let message_view = self.message_view.map(|mv| mv.into_owned());

        ChoiceView::new(
            choices,
            default,
            timeout,
            started_at,
            selected,
            message_view,
        )
    }
}

impl ChoiceView<'_> {
    pub fn choices(&self) -> &Vec<(Cow<'_, ChoiceKey>, Cow<'_, Text>)> {
        &self.choices
    }

    pub fn choices_available(&self) -> Option<&Vec<(Cow<'_, ChoiceKey>, Cow<'_, Text>)>> {
        match &self.message_view {
            None => Some(&self.choices),
            Some(message_view) => message_view.is_completed().then_some(&self.choices),
        }
    }

    pub fn is_available(&self) -> bool {
        match &self.message_view {
            None => true,
            Some(message_view) => message_view.is_completed(),
        }
    }

    pub fn is_selected(&self) -> bool {
        self.selected().is_some()
    }

    pub fn is_selected_manually(&self) -> bool {
        self.selected.is_some()
    }

    pub fn selected(&self) -> Option<&ChoiceKey> {
        self.selected.as_ref().map(|k| k.as_ref()).or_else(|| {
            self.is_expired().then(|| {
                self.default
                    .as_ref()
                    .unwrap_or_else(|| &self.choices.first().expect("At least one choice exists").0)
                    .as_ref()
            })
        })
    }

    pub fn selected_manually(&self) -> Option<&Cow<'_, ChoiceKey>> {
        self.selected.as_ref()
    }

    pub fn message_view(&self) -> &Option<MessageView> {
        &self.message_view
    }

    pub fn has_timeout(&self) -> bool {
        self.timeout.is_some()
    }

    pub fn is_expired(&self) -> bool {
        self.remaining_time().is_zero()
    }

    pub fn remaining_time(&self) -> Duration {
        self.timeout
            .as_ref()
            .and_then(|timeout| {
                let elapsed = self
                    .message_view
                    .as_ref()
                    .and_then(|m| m.completed_at())
                    .unwrap_or(*self.started_at)
                    .elapsed();
                timeout.checked_sub(elapsed)
            })
            .unwrap_or(Duration::MAX)
    }

    pub fn has_message(&self) -> bool {
        self.message_view.is_some()
    }
}
