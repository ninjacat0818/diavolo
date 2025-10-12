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
}

impl DialogueCtxBuilder {
    pub fn actors(mut self, actors: serde_json::Value) -> Self {
        self.actors = Some(Actors(
            actors
                .as_array()
                .unwrap()
                .into_iter()
                .map(|map| {
                    let name = map.get("name").and_then(|v| v.as_str()).unwrap_or_default();
                    Actor {
                        name: ActorName(name.to_owned()),
                    }
                })
                .collect(),
        ));
        self
    }

    pub fn args(mut self, args: serde_json::Value) -> Self {
        self.args = Some(Args::new(args));
        self
    }

    pub fn build(self) -> DialogueCtx {
        let mut ctx = DialogueCtx::default();
        if let Some(actors) = self.actors {
            ctx.actors = actors;
        }
        ctx.args = self.args;
        ctx
    }
}
