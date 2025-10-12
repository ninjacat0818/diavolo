use super::super::error::ValidationError;
use super::Dialogue;
use super::line::{LineType, Owner};
use super::location::Location;

impl Dialogue {
    pub(super) fn validate(&self) -> Result<(), ValidationError> {
        self.validate_owners()?;
        Ok(())
    }

    fn validate_owners(&self) -> Result<(), ValidationError> {
        for (node_key, node) in self.nodes.iter() {
            for (line_idx, line) in node.iter().enumerate() {
                let validate_owner = |owner: &Owner| {
                    let location = Location {
                        node_key: node_key.clone(),
                        line_position: line_idx.into(),
                    };
                    if !self.is_message_allowed() {
                        return Err(ValidationError::MessageNotAllowed { location });
                    } else if !self.actor.is_owner_in_range(**owner) {
                        return Err(ValidationError::OwnerOutOfRange {
                            owner: **owner,
                            max_owner: self.actor.max_owner(),
                            location,
                        });
                    }
                    Ok(())
                };

                match &line.r#type {
                    LineType::Message(message) => {
                        validate_owner(&message.owner)?;
                    }
                    LineType::Choice(choice) => {
                        choice
                            .options
                            .as_ref()
                            .and_then(|opts| opts.message.as_ref())
                            .map(|message| validate_owner(&message.owner))
                            .transpose()?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
