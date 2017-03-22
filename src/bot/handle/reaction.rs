use bot::Bot;
use modelext::MergeIntoMap;

use discord::model::{
    Reaction,
};

use errors::*;

impl Bot {
    pub fn handle_reaction_add(&self, reaction: Reaction) -> Result<()> {
        let server = self.server_by_channel(reaction.channel_id)?;
        let message = server.messages.get(&reaction.message_id);
        let mut map = reaction.into_map()?;
        // TODO: improve infos in map with channel and user
        if let Some(msg) = message {
            msg.clone().merge_into_map_prefix(&mut map, "message_")?;
            let template = server.config.as_ref().and_then(|c| c.reaction_add_cached_msg.as_ref());
            self.log_fmt(server.log_channel, template, &map)?;
        } else {
            let template = server.config.as_ref().and_then(|c| c.reaction_add_uncached_msg.as_ref());
            self.log_fmt(server.log_channel, template, &map)?;
        }
        Ok(())
    }

    pub fn handle_reaction_remove(&self, reaction: Reaction) -> Result<()> {
        let server = self.server_by_channel(reaction.channel_id)?;
        let message = server.messages.get(&reaction.message_id);
        let mut map = reaction.into_map()?;
        // TODO: improve infos in map with channel and user
        if let Some(msg) = message {
            msg.clone().merge_into_map_prefix(&mut map, "message_")?;
            let template = server.config.as_ref().and_then(|c| c.reaction_remove_cached_msg.as_ref());
            self.log_fmt(server.log_channel, template, &map)?;
        } else {
            let template = server.config.as_ref().and_then(|c| c.reaction_remove_uncached_msg.as_ref());
            self.log_fmt(server.log_channel, template, &map)?;
        }
        Ok(())
    }
}
