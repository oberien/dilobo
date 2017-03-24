use std::collections::HashMap;

use bot::Bot;
use modelext::{MergeIntoMap, Diff, ChannelUpdateDiff};

use discord::model::{
    Channel,
};

use errors::*;

impl Bot {
    pub fn handle_channel_create(&mut self, channel: Channel) -> Result<()> {
        if let Channel::Public(channel) = channel {
            self.channels.insert(channel.id, channel.server_id);
            {
                let server = self.server_by_server_mut(channel.server_id)?;
                server.channels.insert(channel.id, channel.clone());
            }
            let server = self.server_by_server(channel.server_id)?;
            let map = channel.into_map()?;
            let template = server.config.as_ref().and_then(|c| c.channel_create_msg.as_ref());
            self.log_fmt(server.log_channel, template, &map)?;
        } else {
            assert!(false, "handle_channel_create: expected public channel: {:?}", channel);
        }
        Ok(())
    }

    pub fn handle_channel_update(&mut self, channel: Channel) -> Result<()> {
        let mut new = None;
        if let Channel::Public(channel) = channel {
            new = Some(channel);
        } else {
            assert!(false, "handle_channel_update: expected public channel: {:?}", channel);
        }
        let channel = new.unwrap();
        let diffs;
        {
            let server = self.server_by_server_mut(channel.server_id)?;
            let old = unwrap!(server.channels.insert(channel.id, channel.clone()));
            diffs = old.diff(&channel)?;
        }
        let server = self.server_by_server(channel.server_id)?;
        let mut map = HashMap::new();
        map.insert("channel_id".to_string(), channel.id.to_string());
        map.insert("channel_name".to_string(), channel.name);
        if diffs.is_empty() {
            let template = server.config.as_ref().and_then(|c| c.channel_update_no_change_msg.as_ref());
            self.log_fmt(server.log_channel, template, &map)?;
            return Ok(());
        }
        for diff in diffs {
            let fmt = match diff {
                ChannelUpdateDiff::Name(..) => server.config.as_ref().and_then(|c| c.channel_update_name_msg.as_ref()),
                ChannelUpdateDiff::UserPermission(..) => server.config.as_ref().and_then(|c| c.channel_update_user_perms_msg.as_ref()),
                ChannelUpdateDiff::RolePermission(..) => server.config.as_ref().and_then(|c| c.channel_update_role_perms_msg.as_ref()),
                ChannelUpdateDiff::Topic(..) => server.config.as_ref().and_then(|c| c.channel_update_topic_msg.as_ref()),
                ChannelUpdateDiff::Position(..) => server.config.as_ref().and_then(|c| c.channel_update_position_msg.as_ref()),
                ChannelUpdateDiff::Bitrate(..) => server.config.as_ref().and_then(|c| c.channel_update_bitrate_msg.as_ref()),
                ChannelUpdateDiff::UserLimit(..) => server.config.as_ref().and_then(|c| c.channel_update_user_limit_msg.as_ref()),
            };
            let mut map = map.clone();
            diff.merge_into_map(&mut map)?;
            self.log_fmt(server.log_channel, fmt, &map)?;
        }
        Ok(())
    }

    pub fn handle_channel_delete(&mut self, channel: Channel) -> Result<()> {
        if let Channel::Public(channel) = channel {
            let channel_id = channel.id;
            let server_id = channel.server_id;
            {
                let server = self.server_by_server(channel.server_id)?;
                let map = channel.into_map()?;
                let template = server.config.as_ref().and_then(|c| c.channel_delete_msg.as_ref());
                self.log_fmt(server.log_channel, template, &map)?;
            }
            unwrap!(self.channels.remove(&channel_id));
            let server = self.server_by_server_mut(server_id)?;
            unwrap!(server.channels.remove(&channel_id));
        } else {
            assert!(false, "handle_channel_delete: expected public channel: {:?}", channel);
        }
        Ok(())
    }
}