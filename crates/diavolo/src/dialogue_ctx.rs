mod actors;
mod args;

pub use actors::*;
pub use args::*;

#[derive(Debug, Default)]
pub struct DialogueCtx {
    actors: Actors,
    args: Option<Args>,
}

impl DialogueCtx {
    pub fn builder() -> DialogueCtxBuilder {
        DialogueCtxBuilder::default()
    }

    pub fn actors(&self) -> &Actors {
        &self.actors
    }

    pub fn actors_count(&self) -> u8 {
        self.actors.len() as u8
    }

    pub fn actor(&self, owner: &dialogue::Owner) -> Option<&Actor> {
        self.actors.get(**owner as usize)
    }

    pub fn args(&self) -> &Option<Args> {
        &self.args
    }

    pub fn parsed_args(
        &mut self,
        dialogue_args: &dialogue::Args,
    ) -> Result<Option<DialogueArgs>, Box<dyn std::error::Error>> {
        match self.args.take() {
            Some(args) => Ok(Some(args.try_to_parsed(dialogue_args)?)),
            None if !dialogue_args.is_empty() => Err("No args provided".into()),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Default)]
pub struct DialogueCtxBuilder {
    actors: Option<Actors>,
    args: Option<Args>,
    system_actor: bool,
}

impl DialogueCtxBuilder {
    pub fn actors(mut self, actors: serde_json::Value) -> Result<Self, Box<dyn std::error::Error>> {
        self.actors.replace(actors.try_into()?);
        Ok(self)
    }

    pub fn args(mut self, args: serde_json::Value) -> Self {
        self.args = Some(Args::new(args));
        self
    }

    pub fn system_actor(mut self, condition: bool) -> Self {
        self.system_actor = condition;
        self
    }

    pub fn build(self) -> DialogueCtx {
        let mut ctx = DialogueCtx::default();
        if let Some(mut actors) = self.actors {
            if self.system_actor {
                actors.insert(0, Actor::system());
            }
            ctx.actors = actors;
        }
        ctx.args = self.args;
        ctx
    }
}
