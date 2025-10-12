mod dialogue;
mod error;

pub mod prelude {
    pub use crate::dialogue::{
        Dialogue,
        actor_info::ActorInfo,
        args::{ArgName, ArgType, ArgVar, Args},
        dialogue_name::DialogueName,
        node::{
            Node,
            line::{Line, LineId, LineIf},
            line_type::*,
        },
        nodes::{NodeKey, Nodes},
    };
}

pub use prelude::*;

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    fn _prelude_playground() {}
}
