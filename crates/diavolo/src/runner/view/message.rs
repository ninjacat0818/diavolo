use dialogue::Text;

use std::borrow::Cow;
use std::time::Instant;

#[derive(Debug, PartialEq, Clone)]
pub struct MessageView<'a> {
    text: Cow<'a, Text>,
    lifecycle: MessageLifecycle,
}

impl<'a> MessageView<'a> {
    pub fn new(text: Cow<'a, Text>, lifecycle: MessageLifecycle) -> Self {
        Self { text, lifecycle }
    }

    pub fn into_owned(self) -> MessageView<'static> {
        MessageView {
            lifecycle: self.lifecycle,
            text: Cow::Owned(self.text.into_owned()),
        }
    }

    pub fn lifecycle(&self) -> &MessageLifecycle {
        &self.lifecycle
    }

    pub fn visible_str(&self) -> &str {
        match self.lifecycle {
            MessageLifecycle::Typing(visible_chars_count) => {
                let (idx, _) = self.text.char_indices().nth(visible_chars_count).unwrap();
                &self.text[..idx]
            }
            MessageLifecycle::Finished | MessageLifecycle::Completed(_) => self.text.as_str(),
        }
    }

    pub fn text(&self) -> &Cow<'_, Text> {
        &self.text
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
