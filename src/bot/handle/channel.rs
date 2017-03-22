use bot::Bot;
use modelext::MergeIntoMap;

use discord::model::{
    Channel,
};

use errors::*;

impl Bot {
    pub fn handle_channel_create(&mut self, channel: Channel) -> Result<()> {
        if let Channel::Public(channel) = channel {
            self.channels.insert(channel.id, channel.server_id);
            let server = self.server_by_channel(channel.id)?;
            let map = channel.into_map()?;
            let template = server.config.as_ref().and_then(|c| c.channel_create_msg.as_ref());
            self.log_fmt(server.log_channel, template, &map)?;
        } else {
            assert!(false, "handle_channel_create: expected public channel: {:?}", channel);
        }
        Ok(())
    }

    pub fn handle_channel_update(&mut self, channel: Channel) -> Result<()> {
        // TODO: calculate diff
        // TODO: update channel in cache
        if let Channel::Public(channel) = channel {
            let server = self.server_by_channel(channel.id)?;
            self.log(server.log_channel, &format!("Channel Changed: {:?}", channel))?;
        } else {
            assert!(false, "handle_channel_update: expected public channel: {:?}", channel);
        }
        Ok(())
    }

    pub fn handle_channel_delete(&mut self, channel: Channel) -> Result<()> {
        if let Channel::Public(channel) = channel {
            let channel_id = channel.id;
            {
                let server = self.server_by_channel(channel.id)?;
                let map = channel.into_map()?;
                let template = server.config.as_ref().and_then(|c| c.channel_delete_msg.as_ref());
                self.log_fmt(server.log_channel, template, &map)?;
            }
            unwrap!(self.channels.remove(&channel_id));
        } else {
            assert!(false, "handle_channel_delete: expected public channel: {:?}", channel);
        }
        Ok(())
    }
}