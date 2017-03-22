use std::collections::HashMap;

use discord::model::{
    Member,
    User,
    UserId,
    Message,
    MessageType,
    MessageDelete,
    Role,
    PublicChannel,
    Reaction,
    ReactionEmoji,
    Emoji,
    MessageUpdate,
    Attachment
};
use discord::model::permissions::{self, Permissions};
use modelext::diff::MessageUpdateDiff;

use errors::*;

pub trait MergeIntoMap {
    fn merge_into_map(self, map: &mut HashMap<String, String>) -> Result<()>;
    fn merge_into_map_prefix(self, map: &mut HashMap<String, String>, prefix: &str) -> Result<()>
            where Self: ::std::marker::Sized {
        let mut new_map = self.into_map()?;
        for (k, v) in new_map.drain() {
            map.insert(prefix.to_string() + &k, v);
        }
        Ok(())
    }
    fn into_map(self) -> Result<HashMap<String, String>> where Self: ::std::marker::Sized {
        let mut map = HashMap::new();
        self.merge_into_map(&mut map)?;
        Ok(map)
    }
    fn into_map_prefix(self, prefix: &str) -> Result<HashMap<String, String>> where Self: ::std::marker::Sized {
        let mut map = HashMap::new();
        self.merge_into_map_prefix(&mut map, prefix)?;
        Ok(map)
    }
}

impl MergeIntoMap for User {
    fn merge_into_map(self, map: &mut HashMap<String, String>) -> Result<()> {
        let User { id, name, discriminator, avatar, bot } = self;
        map.insert("id".to_string(), id.to_string());
        map.insert("name".to_string(), name);
        map.insert("discriminator".to_string(), discriminator.to_string());
        map.insert("avatar".to_string(), avatar.unwrap_or("None".to_string()));
        if bot {
            map.insert("type".to_string(), "bot".to_string());
            map.insert("Type".to_string(), "Bot".to_string());
        } else {
            map.insert("type".to_string(), "member".to_string());
            map.insert("Type".to_string(), "Member".to_string());
        }
        Ok(())
    }
}

impl MergeIntoMap for Member {
    fn merge_into_map(self, map: &mut HashMap<String, String>) -> Result<()> {
        let Member { user, roles, nick, joined_at: time, mute, deaf } = self;
        // TODO: find better solution to provide lists
        map.insert("roles".to_string(), format!("{:?}", roles));
        map.insert("nick".to_string(), nick.unwrap_or("".to_string()));
        map.insert("time".to_string(), time);
        // TODO: find better way to format mute and deaf
        map.insert("mute".to_string(), mute.to_string());
        map.insert("deaf".to_string(), deaf.to_string());
        user.merge_into_map(map)?;
        Ok(())
    }
}

impl MergeIntoMap for Message {
    fn merge_into_map(self, map: &mut HashMap<String, String>) -> Result<()> {
        let Message { id, channel_id, content, nonce, tts, timestamp: time,
            edited_timestamp: edited_time, pinned, kind, author,
            mention_everyone, mentions, mention_roles, reactions, attachments,
            embeds 
        } = self;
        assert!(kind == MessageType::Regular || kind == MessageType::MessagePinned);
        map.insert("id".to_string(), id.to_string());
        map.insert("channel_id".to_string(), channel_id.to_string());
        match kind {
            MessageType::Regular => map.insert("content".to_string(), content.to_string()),
            MessageType::MessagePinned => map.insert("content".to_string(), "<message pinned>".to_string()),
            _ => unreachable!()
        };
        map.insert("nonce".to_string(), nonce.unwrap_or("None".to_string()));
        map.insert("tts".to_string(), tts.to_string());
        map.insert("time".to_string(), time.to_string());
        map.insert("edited_time".to_string(), edited_time.unwrap_or("None".to_string()));
        map.insert("pinned".to_string(), pinned.to_string());
        author.merge_into_map_prefix(map, "author_")?;
        map.insert("mention_everyone".to_string(), mention_everyone.to_string());
        // TODO: find better solution to provide lists
        map.insert("mentions".to_string(), format!("{:?}", mentions));
        // TODO: find better solution to provide lists
        map.insert("mention_roles".to_string(), format!("{:?}", mention_roles));
        // TODO: find better solution to provide lists
        map.insert("reactions".to_string(), format!("{:?}", reactions));
        // TODO: find better solution to provide lists
        map.insert("attachments".to_string(), format!("{:?}", attachments));
        // TODO: find better solution to provide lists
        map.insert("embeds".to_string(), format!("{:?}", embeds));
        Ok(())
    }
}

impl MergeIntoMap for MessageDelete {
    fn merge_into_map(self, map: &mut HashMap<String, String>) -> Result<()> {
        let MessageDelete { channel_id, message_id } = self;
        map.insert("channel_id".to_string(), channel_id.to_string());
        map.insert("message_id".to_string(), message_id.to_string());
        Ok(())
    }
}

impl MergeIntoMap for Role {
    fn merge_into_map(self, map: &mut HashMap<String, String>) -> Result<()> {
        let Role { id, name, color, hoist, managed, position,
                   mentionable, permissions: perms } = self;
        map.insert("id".to_string(), id.to_string());
        map.insert("name".to_string(), name.to_string());
        map.insert("color".to_string(), format!("{:x}", color));
        map.insert("hoist".to_string(), hoist.to_string());
        map.insert("managed".to_string(), managed.to_string());
        map.insert("position".to_string(), position.to_string());
        map.insert("mentionable".to_string(), mentionable.to_string());
        map.insert("perms".to_string(), format!("{:?}", perms));
        perms.merge_into_map_prefix(map, "perm_")?;
        Ok(())
    }
}

impl MergeIntoMap for Permissions {
    fn merge_into_map(self, map: &mut HashMap<String, String>) -> Result<()> {
        map.insert("add_reactions".to_string(), self.contains(permissions::ADD_REACTIONS).to_string());
        map.insert("administrator".to_string(), self.contains(permissions::ADMINISTRATOR).to_string());
        map.insert("attach_files".to_string(), self.contains(permissions::ATTACH_FILES).to_string());
        map.insert("ban_members".to_string(), self.contains(permissions::BAN_MEMBERS).to_string());
        map.insert("change_nicknames".to_string(), self.contains(permissions::CHANGE_NICKNAMES).to_string());
        map.insert("create_invite".to_string(), self.contains(permissions::CREATE_INVITE).to_string());
        map.insert("embed_links".to_string(), self.contains(permissions::EMBED_LINKS).to_string());
        map.insert("external_emojis".to_string(), self.contains(permissions::EXTERNAL_EMOJIS).to_string());
        map.insert("kick_members".to_string(), self.contains(permissions::KICK_MEMBERS).to_string());
        map.insert("manage_channels".to_string(), self.contains(permissions::MANAGE_CHANNELS).to_string());
        map.insert("manage_emojis".to_string(), self.contains(permissions::MANAGE_EMOJIS).to_string());
        map.insert("manage_messages".to_string(), self.contains(permissions::MANAGE_MESSAGES).to_string());
        map.insert("manage_nicknames".to_string(), self.contains(permissions::MANAGE_NICKNAMES).to_string());
        map.insert("manage_roles".to_string(), self.contains(permissions::MANAGE_ROLES).to_string());
        map.insert("manage_server".to_string(), self.contains(permissions::MANAGE_SERVER).to_string());
        map.insert("manage_webhooks".to_string(), self.contains(permissions::MANAGE_WEBHOOKS).to_string());
        map.insert("mention_everyone".to_string(), self.contains(permissions::MENTION_EVERYONE).to_string());
        map.insert("read_history".to_string(), self.contains(permissions::READ_HISTORY).to_string());
        map.insert("read_messages".to_string(), self.contains(permissions::READ_MESSAGES).to_string());
        map.insert("send_messages".to_string(), self.contains(permissions::SEND_MESSAGES).to_string());
        map.insert("send_tts_messages".to_string(), self.contains(permissions::SEND_TTS_MESSAGES).to_string());
        map.insert("voice_connect".to_string(), self.contains(permissions::VOICE_CONNECT).to_string());
        map.insert("voice_deafen_members".to_string(), self.contains(permissions::VOICE_DEAFEN_MEMBERS).to_string());
        map.insert("voice_move_members".to_string(), self.contains(permissions::VOICE_MOVE_MEMBERS).to_string());
        map.insert("voice_mute_members".to_string(), self.contains(permissions::VOICE_MUTE_MEMBERS).to_string());
        map.insert("voice_speak".to_string(), self.contains(permissions::VOICE_SPEAK).to_string());
        map.insert("voice_use_vad".to_string(), self.contains(permissions::VOICE_USE_VAD).to_string());
        Ok(())
    }
}

impl MergeIntoMap for PublicChannel {
    fn merge_into_map(self, map: &mut HashMap<String, String>) -> Result<()> {
        let PublicChannel { id, name, server_id: _, kind, permission_overwrites,
                topic, position, last_message_id, bitrate, user_limit,
                last_pin_timestamp: last_pin_time } = self;
        map.insert("id".to_string(), id.to_string());
        map.insert("name".to_string(), name.to_string());
        // server_id ignored
        map.insert("Type".to_string(), format!("{:?}", kind));
        // TODO: find better solution to provide lists
        map.insert("perms".to_string(), format!("{:?}", permission_overwrites));
        map.insert("topic".to_string(), topic.unwrap_or("None".to_string()));
        map.insert("position".to_string(), position.to_string());
        map.insert("last_message_id".to_string(), last_message_id.map(|id| id.to_string()).unwrap_or("None".to_string()));
        map.insert("bitrate".to_string(), bitrate.map(|b| b.to_string()).unwrap_or("None".to_string()));
        map.insert("user_limit".to_string(), user_limit.map(|limit| limit.to_string()).unwrap_or("None".to_string()));
        map.insert("last_pin_time".to_string(), last_pin_time.unwrap_or("None".to_string()));
        Ok(())
    }
}

impl MergeIntoMap for Reaction {
    fn merge_into_map(self, map: &mut HashMap<String, String>) -> Result<()> {
        let Reaction { channel_id, message_id, user_id, emoji } = self;
        map.insert("channel_id".to_string(), channel_id.to_string());
        map.insert("message_id".to_string(), message_id.to_string());
        map.insert("user_id".to_string(), user_id.to_string());
        emoji.merge_into_map_prefix(map, "emoji_")?;
        Ok(())
    }
}

impl MergeIntoMap for ReactionEmoji {
    fn merge_into_map(self, map: &mut HashMap<String, String>) -> Result<()> {
        match self {
            ReactionEmoji::Unicode(name) => {
                map.insert("name".to_string(), name);
                map.insert("id".to_string(), "None".to_string());
            },
            ReactionEmoji::Custom { name, id } => {
                map.insert("name".to_string(), name);
                map.insert("id".to_string(), id.to_string());
            }
        }
        Ok(())
    }
}

impl MergeIntoMap for Emoji {
    fn merge_into_map(self, map: &mut HashMap<String, String>) -> Result<()> {
        map.insert("id".to_string(), self.id.to_string());
        map.insert("name".to_string(), self.name);
        map.insert("managed".to_string(), self.managed.to_string());
        map.insert("require_colons".to_string(), self.require_colons.to_string());
        // TODO: find better solution to provide lists
        map.insert("roles".to_string(), format!("{:?}", self.roles));
        Ok(())
    }
}

impl MergeIntoMap for MessageUpdate {
    fn merge_into_map(self, map: &mut HashMap<String, String>) -> Result<()> {
        map.insert("debug".to_string(), format!("{:?}", self));
        let MessageUpdate { id, channel_id, kind, content, nonce, tts, pinned,
                timestamp: time, edited_timestamp: edited_time, author,
                mention_everyone, mentions, mention_roles, attachments,
                embeds } = self;
        map.insert("id".to_string(), id.to_string());
        map.insert("channel_id".to_string(), channel_id.to_string());
        map.insert("kind".to_string(), kind.map(|t| format!("{:?}", t)).unwrap_or("None".to_string()));
        map.insert("content".to_string(), content.unwrap_or("None".to_string()));
        map.insert("nonce".to_string(), nonce.unwrap_or("None".to_string()));
        map.insert("tts".to_string(), tts.map(|t| t.to_string()).unwrap_or("None".to_string()));
        map.insert("pinned".to_string(), pinned.map(|t| t.to_string()).unwrap_or("None".to_string()));
        map.insert("time".to_string(), time.unwrap_or("None".to_string()));
        map.insert("edited_time".to_string(), edited_time.unwrap_or("None".to_string()));
        if let Some(author) = author {
            author.merge_into_map_prefix(map, "author_")?;
        } else {
            User { id: UserId(0), name: "None".to_string(), discriminator: 0, avatar: None, bot: false }.merge_into_map_prefix(map, "author_")?;
        }
        map.insert("mention_everyone".to_string(), mention_everyone.map(|t| t.to_string()).unwrap_or("None".to_string()));
        // TODO: find better solution to provide lists
        map.insert("mentions".to_string(), mentions.map(|t| format!("{:?}", t)).unwrap_or("None".to_string()));
        // TODO: find better solution to provide lists
        map.insert("mention_roles".to_string(), mention_roles.map(|t| format!("{:?}", t)).unwrap_or("None".to_string()));
        // TODO: find better solution to provide lists
        map.insert("attachments".to_string(), attachments.map(|t| format!("{:?}", t)).unwrap_or("None".to_string()));
        // TODO: find better solution to provide lists
        map.insert("embeds".to_string(), embeds.map(|t| format!("{:?}", t)).unwrap_or("None".to_string()));
        Ok(())
    }
}

impl MergeIntoMap for Attachment {
    fn merge_into_map(self, map: &mut HashMap<String, String>) -> Result<()> {
        let Attachment { id, filename, url, proxy_url, size, dimensions } = self;
        map.insert("id".to_string(), id);
        map.insert("filename".to_string(), filename);
        map.insert("url".to_string(), url);
        map.insert("proxy_url".to_string(), proxy_url);
        map.insert("size".to_string(), size.to_string());
        let (width, height) = dimensions.map(|(w, h)| (w.to_string(), h.to_string()))
            .unwrap_or(("None".to_string(), "None".to_string()));
        map.insert("width".to_string(), width);
        map.insert("height".to_string(), height);
        Ok(())
    }
}

impl MergeIntoMap for MessageUpdateDiff {
    fn merge_into_map(self, map: &mut HashMap<String, String>) -> Result<()> {
        match self {
            MessageUpdateDiff::Kind(from, to) => {
                map.insert("from".to_string(), format!("{:?}", from));
                map.insert("to".to_string(), format!("{:?}", to));
            },
            MessageUpdateDiff::Content(from, to) => {
                map.insert("from".to_string(), from);
                map.insert("to".to_string(), to);
            },
            MessageUpdateDiff::Nonce(from, to) => {
                map.insert("from".to_string(), from.unwrap_or("None".to_string()));
                map.insert("to".to_string(), to.unwrap_or("None".to_string()));
            },
            MessageUpdateDiff::Tts(from, to) => {
                map.insert("from".to_string(), from.to_string());
                map.insert("to".to_string(), to.to_string());
            },
            MessageUpdateDiff::Pinned => {},
            MessageUpdateDiff::UnPinned => {},
            MessageUpdateDiff::EditedTimestamp(from, to) => {
                map.insert("from".to_string(), from.unwrap_or("None".to_string()));
                map.insert("to".to_string(), to);
            },
            MessageUpdateDiff::MentionEveryone(from, to) => {
                map.insert("from".to_string(), from.to_string());
                map.insert("to".to_string(), to.to_string());
            },
            MessageUpdateDiff::MentionAdded(user) => {
                user.merge_into_map(map)?;
            },
            MessageUpdateDiff::MentionRemoved(user) => {
                user.merge_into_map(map)?;
            },
            MessageUpdateDiff::MentionRoleAdded(role_id) => {
                map.insert("id".to_string(), role_id.to_string());
            },
            MessageUpdateDiff::MentionRoleRemoved(role_id) => {
                map.insert("id".to_string(), role_id.to_string());
            },
            MessageUpdateDiff::AttachmentAdded(attachment) => {
                attachment.merge_into_map(map)?;
            },
            MessageUpdateDiff::AttachmentRemoved(attachment) => {
                attachment.merge_into_map(map)?;
            },
            MessageUpdateDiff::EmbedsAdded(value) => {
                map.insert("value".to_string(), value.to_string());
            },
            MessageUpdateDiff::EmbedsRemoved(value) => {
                map.insert("value".to_string(), value.to_string());
            },
        }
        Ok(())
    }
}
