pub mod call;
pub mod choice;
pub mod confirm;
pub mod eval;
pub mod exit;
pub mod goto;
pub mod message;
pub mod r#return;
pub mod texts;

pub use call::*;
pub use choice::*;
pub use confirm::*;
pub use eval::*;
pub use exit::*;
pub use goto::*;
pub use message::*;
pub use r#return::*;
pub use texts::*;

use serde::Serialize;

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(untagged)]
pub enum LineType {
    Message(Message),
    Confirm(Confirm),
    Choice(Choice),
    // Input,
    Eval(Eval),
    // Event,
    // Use,
    Goto(Goto),
    Call(Call),
    Return(Return),
    Exit(Exit),
}
