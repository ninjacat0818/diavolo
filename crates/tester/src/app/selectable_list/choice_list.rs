use diavolo::dialogue::ChoiceKey;
use ratatui::text::Line;
use ratatui::widgets::ListItem;

pub struct ChoiceList {
    pub items: Vec<ChoiceItem>,
}

impl ChoiceList {
    pub fn selected_key(&self, idx: usize) -> &ChoiceKey {
        self.items
            .get(idx)
            .map(|item| &item.key)
            .expect("There should be a selected choice")
    }
}

impl Into<Vec<ListItem<'_>>> for &ChoiceList {
    fn into(self) -> Vec<ListItem<'static>> {
        self.items
            .iter()
            .map(ListItem::from)
            .collect::<Vec<_>>()
    }
}

impl<'a> FromIterator<(&'a ChoiceKey, &'a str)> for ChoiceList {
    fn from_iter<T: IntoIterator<Item = (&'a ChoiceKey, &'a str)>>(iter: T) -> Self {
        let items = iter
            .into_iter()
            .map(|(key, text)| ChoiceItem::new(key.clone(), text.to_string()))
            .collect::<Vec<_>>();

        Self { items }
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
