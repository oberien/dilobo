use bot::Bot;
use modelext::MergeIntoMap;

use discord::Result;
use discord::model::{
    Channel,
};

impl Bot {
    pub fn handle_channel_create(&mut self, channel: Channel) -> Result<()> {
        // TODO: add channel to cache
        if let Channel::Public(channel) = channel {
            self.channels.insert(channel.id, channel.server_id);
            let server = self.server_by_channel(channel.id);
            let map = channel.into_map();
            self.log_fmt(server.log_channel, server.config.channel_create_msg.as_ref(), &map)?;
        }
        // TODO: Return error if channel is not public once error_chain is in use
        Ok(())
    }

    pub fn handle_channel_update(&mut self, channel: Channel) -> Result<()> {
        // TODO: calculate diff
        // TODO: update channel in cache
        if let Channel::Public(channel) = channel {
            let server = self.server_by_channel(channel.id);
            self.log(server.log_channel, &format!("Channel Changed: {:?}", channel))?;
        }
        // TODO: Return error if channel is not public once error_chain is in use
        Ok(())
    }

    pub fn handle_channel_delete(&mut self, channel: Channel) -> Result<()> {
        // TODO: remove channel from cache
        if let Channel::Public(channel) = channel {
            let channel_id = channel.id;
            {
                let server = self.server_by_channel(channel.id);
                let map = channel.into_map();
                self.log_fmt(server.log_channel, server.config.channel_delete_msg.as_ref(), &map)?;
            }
            self.channels.remove(&channel_id);
        }
        // TODO: Return error if channel is not public once error_chain is in use
        Ok(())
    }
}