pub(crate) mod choice;
pub(crate) mod message;

use choice::ChoiceView;
use message::MessageView;

use dialogue::{ChoiceKey, Text};

#[derive(Debug, PartialEq, Clone)]
pub enum View<'d> {
    None,
    Terminated,
    Message(MessageView<'d>),
    Choice(ChoiceView<'d>),
}

impl Default for View<'static> {
    fn default() -> Self {
        View::None
    }
}

impl View<'_> {
    pub fn message(&self) -> Option<&str> {
        match self {
            View::Message(message_view) => Some(message_view.visible_str()),
            View::Choice(choice_view) => choice_view
                .message_view()
                .as_ref()
                .map(|message_view| message_view.visible_str()),
            _ => None,
        }
    }

    pub fn choices(&self) -> Option<Vec<(ChoiceKey, Text)>> {
        match self {
            View::Choice(choice_view) => choice_view.choices_available().map(|choices| {
                choices
                    .into_iter()
                    .map(|(k, v)| (k.as_ref().clone(), v.as_ref().clone()))
                    .collect()
            }),
            _ => None,
        }
    }
}

impl View<'_> {
    pub fn has_message(&self) -> bool {
        match self {
            View::Message(_) => true,
            View::Choice(choice_view) => choice_view.has_message(),
            _ => false,
        }
    }

    pub fn has_message_finished(&self) -> bool {
        match self {
            View::Message(message_view) => message_view.is_finished(),
            View::Choice(choice_view) => choice_view
                .message_view()
                .as_ref()
                .map_or(false, |message_view| message_view.is_finished()),
            _ => false,
        }
    }

    pub fn is_message(&self) -> bool {
        matches!(self, View::Message(_))
    }

    pub fn is_choice(&self) -> bool {
        matches!(self, View::Choice(_))
    }

    pub fn is_choice_available(&self) -> bool {
        match self {
            View::Choice(choice_view) => choice_view.is_available(),
            _ => false,
        }
    }

    pub fn is_terminated(&self) -> bool {
        matches!(self, View::Terminated)
    }
}

impl View<'_> {
    pub fn into_owned(self) -> View<'static> {
        match self {
            View::None => View::None,
            View::Terminated => View::Terminated,
            View::Message(message_view) => View::Message(message_view.into_owned()),
            View::Choice(choice_view) => View::Choice(choice_view.into_owned()),
        }
    }
}
