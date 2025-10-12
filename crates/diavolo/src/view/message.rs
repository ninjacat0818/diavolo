use crate::dialogue_ctx::{Actor, ViewActor};
use dialogue::{Text, Texts};

use std::borrow::Cow;
use std::time::Instant;

#[derive(Debug, PartialEq, Clone)]
pub struct MessageView<'a> {
    actor: Cow<'a, Actor>,
    view_actor: ViewActor<'a>,
    text: Cow<'a, Text>,
    texts: Cow<'a, Texts>,
    lifecycle: MessageLifecycle,
}

impl<'a> MessageView<'a> {
    pub fn new(
        actor: &'a Actor,
        view_actor: ViewActor<'a>,
        text: &'a Text,
        texts: &'a Texts,
        lifecycle: MessageLifecycle,
    ) -> Self {
        Self {
            actor: Cow::Borrowed(actor),
            view_actor,
            text: Cow::Borrowed(text),
            texts: Cow::Borrowed(texts),
            lifecycle,
        }
    }

    pub fn into_owned(self) -> MessageView<'static> {
        MessageView {
            actor: Cow::Owned(self.actor.into_owned()),
            view_actor: self.view_actor.into_owned(),
            text: Cow::Owned(self.text.into_owned()),
            texts: Cow::Owned(self.texts.into_owned()),
            lifecycle: self.lifecycle,
        }
    }

    pub fn actor(&self) -> &Actor {
        &self.actor
    }

    pub fn view_actor(&self) -> &ViewActor {
        &self.view_actor
    }

    pub fn visible_str(&self) -> &str {
        match self.lifecycle {
            MessageLifecycle::Typing(visible_chars_count) => {
                let text = &self.text;
                text.char_indices()
                    .nth(visible_chars_count)
                    .map(|(idx, _)| &text[..idx])
                    .expect(
                        "visible chars count should be less than or equal to the total chars count",
                    )
            }
            MessageLifecycle::Finished | MessageLifecycle::Completed(_) => &self.text.as_str(),
        }
    }

    pub fn text(&self) -> &Text {
        &self.text
    }

    pub fn lifecycle(&self) -> &MessageLifecycle {
        &self.lifecycle
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.lifecycle, MessageLifecycle::Finished)
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.lifecycle, MessageLifecycle::Completed(_))
    }

    pub fn completed_at(&self) -> Option<Instant> {
        match self.lifecycle {
            MessageLifecycle::Completed(completed_at) => Some(completed_at),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum MessageLifecycle {
    Typing(usize),
    Finished,
    Completed(Instant),
}
