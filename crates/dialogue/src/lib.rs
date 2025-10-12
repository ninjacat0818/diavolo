mod dialogue;
mod error;

pub use error::Error;

pub mod prelude {
    pub use crate::dialogue::{
        Dialogue, actor_info::*, args::*, dialogue_name::*, line::*, location::*, node::*, nodes::*,
    };
}

pub use prelude::*;

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    fn _prelude_playground() {}
}
