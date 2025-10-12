use ratatui::text::Line;
use ratatui::widgets::ListItem;

pub struct ConfirmList {
    pub yes: String,
    pub no: String,
}

impl Default for ConfirmList {
    fn default() -> Self {
        Self {
            yes: "Yes".to_string(),
            no: "No".to_string(),
        }
    }
}

impl ConfirmList {
    pub fn is_approved(&self, selected_idx: usize) -> bool {
        match selected_idx {
            0 => true,
            1 => false,
            _ => unreachable!(),
        }
    }
}

impl Into<Vec<ListItem<'_>>> for &ConfirmList {
    fn into(self) -> Vec<ListItem<'static>> {
        [&self.yes, &self.no]
            .into_iter()
            .map(|text| ListItem::from(Line::raw(text.clone())))
            .collect::<Vec<_>>()
    }
}
