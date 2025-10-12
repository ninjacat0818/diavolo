use dialogue::{ConfirmResponse, Text};

use super::message::{MessageLifecycle, MessageView};

use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
pub struct ConfirmView<'a> {
    pub(crate) confirmed: bool,
    pub(crate) responses: Option<(Cow<'a, Text>, Cow<'a, Text>)>,
    pub(crate) response_texts: Option<Cow<'a, ConfirmResponse>>,
    pub(crate) message_view: MessageView<'a>,
}

impl<'a> std::ops::Deref for ConfirmView<'a> {
    type Target = MessageView<'a>;

    fn deref(&self) -> &Self::Target {
        &self.message_view
    }
}

impl<'a> ConfirmView<'a> {
    pub fn new(
        confirmed: bool,
        responses: Option<(&'a Text, &'a Text)>,
        response_texts: Option<&'a ConfirmResponse>,
        message_view: MessageView<'a>,
    ) -> Self {
        Self {
            confirmed,
            responses: responses.map(|(yes, no)| (Cow::Borrowed(yes), Cow::Borrowed(no))),
            response_texts: response_texts.map(Cow::Borrowed),
            message_view,
        }
    }

    pub fn into_owned(self) -> ConfirmView<'static> {
        ConfirmView {
            confirmed: self.confirmed,
            responses: self
                .responses
                .map(|(yes, no)| (Cow::Owned(yes.into_owned()), Cow::Owned(no.into_owned()))),
            response_texts: self.response_texts.map(|rt| Cow::Owned(rt.into_owned())),
            message_view: self.message_view.into_owned(),
        }
    }

    pub fn responses(&'a self) -> Option<(&'a str, &'a str)> {
        self.responses
            .as_ref()
            .map(|(yes, no)| (yes.as_str(), no.as_str()))
    }

    pub fn is_available(&self) -> bool {
        !self.confirmed
            && matches!(
                self.message_view.lifecycle(),
                MessageLifecycle::Completed(_)
            )
    }

    pub fn is_confirmed(&self) -> bool {
        self.confirmed
    }
}
