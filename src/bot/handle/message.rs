use std::collections::HashMap;

use bot::Bot;
use modelext::{MergeIntoMap, Diff, MessageUpdateDiff};

use discord::model::{
    Message,
    MessageUpdate,
    User,
    UserId,
    MessageDelete,
    MessageDeleteBulk,
};
use strfmt::strfmt;

use errors::*;

impl Bot {
    pub fn handle_message_create(&mut self, msg: Message) -> Result<()> {
        {
            let mut server = self.server_by_channel_mut(msg.channel_id)?;
            server.messages.insert(msg.id, msg.clone());
        }
        let server = self.server_by_channel(msg.channel_id)?;
        // ignore new messages in log channel which we have created
        if server.log_channel.is_some() && msg.channel_id != server.log_channel.unwrap() || msg.author.id != self.user.id {
            let map = msg.into_map()?;
            let template = server.config.as_ref().and_then(|c| c.message_create_msg.as_ref());
            self.log_fmt(server.log_channel, template, &map)?;
        }
        Ok(())
    }

    pub fn handle_message_update(&mut self, update: MessageUpdate) -> Result<()> {
        let log_channel;
        let cached_author;
        let diffs;
        {
            let server = self.server_by_channel(update.channel_id)?;
            log_channel = server.log_channel;
            let message = server.messages.get(&update.id);
            // If the message's channel is the log_channel and it has something
            // embedded, we must return early as it would create an infinite
            // embed-update loop.
            if log_channel.is_some() && log_channel.unwrap() == update.channel_id && update.embeds != None {
                return Ok(());
            }
            if let None = message {
                let map = update.into_map()?;
                let template = server.config.as_ref().and_then(|c| c.message_update_uncached_msg.as_ref());
                self.log_fmt(log_channel, template, &map)?;
                return Ok(());
            }
            let msg = message.unwrap();
            cached_author = msg.author.clone();
            diffs = msg.diff(&update)?;
        }
        let mut map = HashMap::new();
        cached_author.merge_into_map_prefix(&mut map, "cached_author_")?;
        map.insert("message_id".to_string(), update.id.to_string());
        map.insert("channel_id".to_string(), update.channel_id.to_string());
        if let Some(ref author) = update.author {
            author.clone().merge_into_map_prefix(&mut map, "author_")?;
        } else {
            User { id: UserId(0), name: "None".to_string(), discriminator: 0, avatar: None, bot: false }.merge_into_map_prefix(&mut map, "author_")?;
        }
        for diff in diffs {
            {
                let server = self.server_by_channel_mut(update.channel_id)?;
                let message = server.messages.get_mut(&update.id).unwrap();
                diff.apply(message)?;
            }
            let server = self.server_by_channel(update.channel_id)?;
            let fmt = match diff {
                MessageUpdateDiff::Kind(..) => server.config.as_ref().and_then(|c| c.message_update_kind_msg.as_ref()),
                MessageUpdateDiff::Content(..) => server.config.as_ref().and_then(|c| c.message_update_content_msg.as_ref()),
                MessageUpdateDiff::Nonce(..) => server.config.as_ref().and_then(|c| c.message_update_nonce_msg.as_ref()),
                MessageUpdateDiff::Tts(..) => server.config.as_ref().and_then(|c| c.message_update_tts_msg.as_ref()),
                MessageUpdateDiff::Pinned => server.config.as_ref().and_then(|c| c.message_update_pinned_msg.as_ref()),
                MessageUpdateDiff::UnPinned => server.config.as_ref().and_then(|c| c.message_update_unpinned_msg.as_ref()),
                MessageUpdateDiff::EditedTimestamp(..) => server.config.as_ref().and_then(|c| c.message_update_edited_time_msg.as_ref()),
                MessageUpdateDiff::MentionEveryone(..) => server.config.as_ref().and_then(|c| c.message_update_mention_everyone_msg.as_ref()),
                MessageUpdateDiff::MentionAdded(..) => server.config.as_ref().and_then(|c| c.message_update_mention_added_msg.as_ref()),
                MessageUpdateDiff::MentionRemoved(..) => server.config.as_ref().and_then(|c| c.message_update_mention_removed_msg.as_ref()),
                MessageUpdateDiff::MentionRoleAdded(..) => server.config.as_ref().and_then(|c| c.message_update_mention_role_added_msg.as_ref()),
                MessageUpdateDiff::MentionRoleRemoved(..) => server.config.as_ref().and_then(|c| c.message_update_mention_role_removed_msg.as_ref()),
                MessageUpdateDiff::AttachmentAdded(..) => server.config.as_ref().and_then(|c| c.message_update_attachment_added_msg.as_ref()),
                MessageUpdateDiff::AttachmentRemoved(..) => server.config.as_ref().and_then(|c| c.message_update_attachment_removed_msg.as_ref()),
                MessageUpdateDiff::EmbedsAdded(..) => server.config.as_ref().and_then(|c| c.message_update_embeds_added_msg.as_ref()),
                MessageUpdateDiff::EmbedsRemoved(..) => server.config.as_ref().and_then(|c| c.message_update_embeds_removed_msg.as_ref()),
            };
            let mut map = map.clone();
            diff.merge_into_map(&mut map)?;
            self.log_fmt(log_channel, fmt, &map)?;
        }
        Ok(())
    }

    pub fn handle_message_delete(&mut self, del: MessageDelete) -> Result<()> {
        let message;
        {
            let server = self.server_by_channel_mut(del.channel_id)?;
            message = server.messages.remove(&del.message_id);
        }
        let server = self.server_by_channel(del.channel_id)?;
        if let Some(msg) = message {
            let map = msg.clone().into_map()?;
            let template = server.config.as_ref().and_then(|c| c.message_delete_cached_msg.as_ref());
            self.log_fmt(server.log_channel, template, &map)?;
        } else {
            let map = del.into_map()?;
            let template = server.config.as_ref().and_then(|c| c.message_delete_uncached_msg.as_ref());
            self.log_fmt(server.log_channel, template, &map)?;
        }
        Ok(())
    }

    pub fn handle_message_delete_bulk(&mut self, del: MessageDeleteBulk) -> Result<()> {
        {
            let server = self.server_by_channel(del.channel_id)?;
            let mut map = HashMap::new();
            map.insert("channel_id".to_string(), del.channel_id.to_string());
            map.insert("count".to_string(), del.ids.len().to_string());
            let template = server.config.as_ref().and_then(|c| c.message_delete_bulk_msg.as_ref());
            self.log_fmt(server.log_channel, template, &map)?;
        }
        let mut text = String::new();
        for message_id in del.ids {
            let cached;
            {
                let server = self.server_by_channel_mut(del.channel_id)?;
                cached = server.messages.remove(&message_id);
            }
            let server = self.server_by_channel(del.channel_id)?;
            let line = match cached {
                Some(msg) => server.config.as_ref().and_then(|c| c.message_delete_cached_msg.as_ref())
                        .map(|fmt| msg.into_map().and_then(|map| strfmt(fmt, &map)
                            .map_err(|err| ErrorKind::FormatError(server.id, err).into()))),
                None => {
                    let del = MessageDelete {
                        channel_id: del.channel_id,
                        message_id: message_id,
                    };
                    server.config.as_ref().and_then(|c| c.message_delete_uncached_msg.as_ref())
                        .map(|fmt| del.into_map().and_then(|map| strfmt(fmt, &map)
                            .map_err(|err| ErrorKind::FormatError(server.id, err).into())))
                }
            };
            if let Some(line) = line {
                let line = line?;
                if text.len() + line.len() > 2000 {
                    let server = self.server_by_channel(del.channel_id)?;
                    self.log(server.log_channel, &text)?;
                    text = line;
                } else {
                    text += "\n";
                    text += &line;
                }
            }
        }
        let server = self.server_by_channel(del.channel_id)?;
        self.log(server.log_channel, &text)?;
        Ok(())
    }
}