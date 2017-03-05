use std::collections::HashMap;

use discord::model::{
    Member,
    User,
    Message,
    MessageType,
};

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
