use std::collections::HashMap;

use bot::Bot;
use modelext::{MergeIntoMap, Diff, MessageUpdateDiff};

use discord::Result;
use discord::model::{
    Message,
    MessageUpdate,
    User,
    UserId,
    MessageDelete,
    MessageDeleteBulk,
};
use strfmt::strfmt;

impl Bot {
    pub fn handle_message_create(&mut self, msg: Message) -> Result<()> {
        {
            let mut server = self.server_by_channel_mut(msg.channel_id);
            server.messages.insert(msg.id, msg.clone());
        }
        let server = self.server_by_channel(msg.channel_id);
        // ignore new messages in log channel which we have created
        if msg.channel_id != server.log_channel || msg.author.id != self.user.id {
            let map = msg.into_map();
            self.log_fmt(&server, server.config.message_create_msg.as_ref(), &map)?;
        }
        Ok(())
    }

    pub fn handle_message_update(&self, update: MessageUpdate) -> Result<()> {
        // TODO: update cached message
        let server = self.server_by_channel(update.channel_id);
        let message = server.messages.get(&update.id);
        // If the message's channel is the log_channel and it has something
        // embedded, we must return early as it would create an infinite
        // embed-update loop.
        if server.log_channel == update.channel_id && update.embeds != None {
            return Ok(());
        }
        if let Some(msg) = message {
            let mut map = HashMap::new();
            msg.author.clone().merge_into_map_prefix(&mut map, "cached_author_");
            map.insert("message_id".to_string(), update.id.to_string());
            map.insert("channel_id".to_string(), update.channel_id.to_string());
            if let Some(ref author) = update.author {
                author.clone().merge_into_map_prefix(&mut map, "author_");
            } else {
                User { id: UserId(0), name: "None".to_string(), discriminator: 0, avatar: None, bot: false }.merge_into_map_prefix(&mut map, "author_");
            }
            let diffs = msg.diff(&update);
            for diff in diffs {
                let fmt = match diff {
                    MessageUpdateDiff::Kind(..) => server.config.message_update_kind_msg.as_ref(),
                    MessageUpdateDiff::Content(..) => server.config.message_update_content_msg.as_ref(),
                    MessageUpdateDiff::Nonce(..) => server.config.message_update_nonce_msg.as_ref(),
                    MessageUpdateDiff::Tts(..) => server.config.message_update_tts_msg.as_ref(),
                    MessageUpdateDiff::Pinned => server.config.message_update_pinned_msg.as_ref(),
                    MessageUpdateDiff::UnPinned => server.config.message_update_unpinned_msg.as_ref(),
                    MessageUpdateDiff::EditedTimestamp(..) => server.config.message_update_edited_time_msg.as_ref(),
                    MessageUpdateDiff::MentionEveryone(..) => server.config.message_update_mention_everyone_msg.as_ref(),
                    MessageUpdateDiff::MentionAdded(..) => server.config.message_update_mention_added_msg.as_ref(),
                    MessageUpdateDiff::MentionRemoved(..) => server.config.message_update_mention_removed_msg.as_ref(),
                    MessageUpdateDiff::MentionRoleAdded(..) => server.config.message_update_mention_role_added_msg.as_ref(),
                    MessageUpdateDiff::MentionRoleRemoved(..) => server.config.message_update_mention_role_removed_msg.as_ref(),
                    MessageUpdateDiff::AttachmentAdded(..) => server.config.message_update_attachment_added_msg.as_ref(),
                    MessageUpdateDiff::AttachmentRemoved(..) => server.config.message_update_attachment_removed_msg.as_ref(),
                    MessageUpdateDiff::EmbedsAdded(..) => server.config.message_update_embeds_added_msg.as_ref(),
                    MessageUpdateDiff::EmbedsRemoved(..) => server.config.message_update_embeds_removed_msg.as_ref(),
                };
                let mut map = map.clone();
                diff.merge_into_map(&mut map);
                self.log_fmt(&server, fmt, &map)?;
            }
        } else {
            let map = update.into_map();
            self.log_fmt(&server, server.config.message_update_uncached_msg.as_ref(), &map)?;
        }
        Ok(())
    }

    pub fn handle_message_delete(&mut self, del: MessageDelete) -> Result<()> {
        let message;
        {
            let server = self.server_by_channel_mut(del.channel_id);
            message = server.messages.remove(&del.message_id);
        }
        let server = self.server_by_channel(del.channel_id);
        if let Some(msg) = message {
            let map = msg.clone().into_map();
            self.log_fmt(&server, server.config.message_delete_cached_msg.as_ref(), &map)?;
        } else {
            let map = del.into_map();
            self.log_fmt(&server, server.config.message_delete_uncached_msg.as_ref(), &map)?;
        }
        Ok(())
    }

    pub fn handle_message_delete_bulk(&mut self, del: MessageDeleteBulk) -> Result<()> {
        {
            let server = self.server_by_channel(del.channel_id);
            let mut map = HashMap::new();
            map.insert("channel_id".to_string(), del.channel_id.to_string());
            map.insert("count".to_string(), del.ids.len().to_string());
            self.log_fmt(&server, server.config.message_delete_bulk_msg.as_ref(), &map)?;
        }
        let mut text = String::new();
        for message_id in del.ids {
            let cached;
            {
                let server = self.server_by_channel_mut(del.channel_id);
                cached = server.messages.remove(&message_id);
            }
            let server = self.server_by_channel(del.channel_id);
            let line = match cached {
                // TODO: use error_chain instead of unwrap
                Some(msg) => server.config.message_delete_cached_msg
                    .as_ref().map(|fmt| strfmt(fmt, &msg.into_map()).unwrap()),
                None => {
                    let del = MessageDelete {
                        channel_id: del.channel_id,
                        message_id: message_id,
                    };
                    server.config.message_delete_uncached_msg
                        .as_ref().map(|fmt| strfmt(fmt, &del.into_map()).unwrap())
                }
            };
            if let Some(line) = line {
                if text.len() + line.len() > 2000 {
                    let server = self.server_by_channel(del.channel_id);
                    self.log(&server, &text)?;
                    text = line;
                } else {
                    text += "\n";
                    text += &line;
                }
            }
        }
        let server = self.server_by_channel(del.channel_id);
        self.log(&server, &text)?;
        Ok(())
    }
}