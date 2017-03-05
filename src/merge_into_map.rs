use std::collections::HashMap;

use discord::model::{
    Member,
    User,
    Message,
    MessageType,
    MessageDelete,
    Role,
};
use discord::model::permissions::{self, Permissions};

pub trait MergeIntoMap {
    fn merge_into_map(self, map: &mut HashMap<String, String>);
    fn merge_into_map_prefix(self, map: &mut HashMap<String, String>, prefix: &str)
            where Self: ::std::marker::Sized {
        let mut new_map = self.into_map();
        for (k, v) in new_map.drain() {
            map.insert(prefix.to_string() + &k, v);
        }
    }
    fn into_map(self) -> HashMap<String, String> where Self: ::std::marker::Sized {
        let mut map = HashMap::new();
        self.merge_into_map(&mut map);
        map
    }
}

impl MergeIntoMap for User {
    fn merge_into_map(self, map: &mut HashMap<String, String>) {
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
    }
}

impl MergeIntoMap for Member {
    fn merge_into_map(self, map: &mut HashMap<String, String>) {
        let Member { user, roles, nick, joined_at: time, mute, deaf } = self;
        // TODO: find better solution to provide lists
        map.insert("roles".to_string(), format!("{:?}", roles));
        map.insert("nick".to_string(), nick.unwrap_or("".to_string()));
        map.insert("time".to_string(), time);
        // TODO: find better way to format mute and deaf
        map.insert("mute".to_string(), mute.to_string());
        map.insert("deaf".to_string(), deaf.to_string());
        user.merge_into_map(map);
    }
}

impl MergeIntoMap for Message {
    fn merge_into_map(self, map: &mut HashMap<String, String>) {
        let Message { id, channel_id, content, nonce, tts, timestamp: time,
            edited_timestamp: edited_time, pinned, kind, author,
            mention_everyone, mentions, mention_roles, reactions, attachments,
            embeds 
        } = self;
        assert_eq!(kind, MessageType::Regular);
        map.insert("id".to_string(), id.to_string());
        map.insert("channel_id".to_string(), channel_id.to_string());
        map.insert("content".to_string(), content.to_string());
        map.insert("nonce".to_string(), nonce.unwrap_or("None".to_string()));
        map.insert("tts".to_string(), tts.to_string());
        map.insert("time".to_string(), time.to_string());
        map.insert("edited_time".to_string(), edited_time.unwrap_or("None".to_string()));
        map.insert("pinned".to_string(), pinned.to_string());
        author.merge_into_map_prefix(map, "author_");
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
    }
}

impl MergeIntoMap for MessageDelete {
    fn merge_into_map(self, map: &mut HashMap<String, String>) {
        let MessageDelete { channel_id, message_id } = self;
        map.insert("channel_id".to_string(), channel_id.to_string());
        map.insert("message_id".to_string(), message_id.to_string());
    }
}

impl MergeIntoMap for Role {
    fn merge_into_map(self, map: &mut HashMap<String, String>) {
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
        perms.merge_into_map_prefix(map, "perm_");
    }
}

impl MergeIntoMap for Permissions {
    fn merge_into_map(self, map: &mut HashMap<String, String>) {
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
    }
}
