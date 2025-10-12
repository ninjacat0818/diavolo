mod dialogue;

pub mod prelude {
    pub use crate::dialogue::{
        Dialogue,
        actor_info::ActorInfo,
        args::{ArgName, ArgType, Args},
        dialogue_name::DialogueName,
        node::{
            Node,
            line::{Line, LineIf},
            line_type::*,
        },
        nodes::{NodeKey, Nodes},
    };
}

pub use prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn _prelude_playground() {}
}
