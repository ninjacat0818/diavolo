use diavolo::dialogue::{ChoiceKey, Text};
use ratatui::text::Line;
use ratatui::widgets::{ListItem, ListState};

pub struct ChoiceList {
    pub items: Vec<ChoiceItem>,
    pub state: ListState,
}

impl ChoiceList {
    pub fn selected_key(&self) -> &ChoiceKey {
        self.state
            .selected()
            .and_then(|selected_idx| self.items.get(selected_idx).map(|item| &item.key))
            .expect("There should be a selected choice")
    }
}

impl FromIterator<(ChoiceKey, Text)> for ChoiceList {
    fn from_iter<T: IntoIterator<Item = (ChoiceKey, Text)>>(iter: T) -> Self {
        let items = iter
            .into_iter()
            .map(|(key, text)| ChoiceItem::new(key.clone(), text.to_string()))
            .collect::<Vec<_>>();

        Self {
            items,
            state: ListState::default().with_selected(Some(0)),
        }
    }
}

pub struct ChoiceItem {
    pub key: ChoiceKey,
    text: String,
}

impl ChoiceItem {
    fn new(key: ChoiceKey, text: String) -> Self {
        Self { key, text }
    }
}

impl From<&ChoiceItem> for ListItem<'_> {
    fn from(value: &ChoiceItem) -> Self {
        let line = Line::raw(value.text.clone());
        ListItem::new(line)
    }
}
