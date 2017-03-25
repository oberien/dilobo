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
        let user = unwrap!(server.members.get(&reaction.user_id)).clone();
        let channel = unwrap!(server.channels.get(&reaction.channel_id)).clone();
        let mut map = reaction.into_map()?;
        user.merge_into_map_prefix(&mut map, "user_")?;
        channel.merge_into_map_prefix(&mut map, "channel_")?;
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
        let user = unwrap!(server.members.get(&reaction.user_id)).clone();
        let channel = unwrap!(server.channels.get(&reaction.channel_id)).clone();
        let mut map = reaction.into_map()?;
        user.merge_into_map_prefix(&mut map, "user_")?;
        channel.merge_into_map_prefix(&mut map, "channel_")?;
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
