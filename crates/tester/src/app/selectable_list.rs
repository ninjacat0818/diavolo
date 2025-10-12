mod choice_list;
mod confirm_list;

use choice_list::ChoiceList;
use confirm_list::ConfirmList;

use ratatui::widgets::ListState;

use diavolo::Selectable;

pub struct SelectableList {
    pub r#type: ListType,
    pub state: ListState,
}

impl SelectableList {
    pub fn new(r#type: ListType) -> Self {
        Self {
            r#type,
            state: ListState::default().with_selected(Some(0)),
        }
    }
}

impl From<Selectable<'_>> for SelectableList {
    fn from(value: Selectable<'_>) -> Self {
        match value {
            Selectable::Confirm(confirm) => {
                let confirm_list = confirm
                    .map(|(yes, no)| ConfirmList {
                        yes: yes.to_string(),
                        no: no.to_string(),
                    })
                    .unwrap_or_default();
                Self::new(ListType::Confirm(confirm_list))
            }
            Selectable::Choice(choices) => {
                let choice_list = choices.into_iter().collect::<ChoiceList>();
                Self::new(ListType::Choice(choice_list))
            }
        }
    }
}

pub enum ListType {
    Confirm(ConfirmList),
    Choice(ChoiceList),
}
