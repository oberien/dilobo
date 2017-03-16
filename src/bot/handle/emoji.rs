use std::collections::HashMap;

use bot::Bot;
use modelext::{MergeIntoMap, Diff, EmojisUpdateDiff};

use discord::Result;
use discord::model::{
    ServerId,
    Emoji,
};

impl Bot {
    pub fn handle_server_emojis_update(&mut self, server_id: ServerId, update: Vec<Emoji>) -> Result<()> {
        let mut diffs;
        {
            let server = self.server_by_server_mut(server_id);
            let mut emojis = &mut server.emojis;
            diffs = emojis.diff(&update);
            for diff in diffs.iter() {
                diff.apply(&mut emojis);
            }
        }
        let server = self.server_by_server(server_id);
        for diff in diffs.drain(..) {
            match diff {
                EmojisUpdateDiff::EmojiAdded(emoji) => {
                    let map = emoji.into_map_prefix("emoji_");
                    let template = server.config.as_ref().and_then(|c| c.server_emoji_add_msg.as_ref());
                    self.log_fmt(server.log_channel, template, &map)?;
                },
                EmojisUpdateDiff::EmojiRemoved(emoji) => {
                    let map = emoji.into_map_prefix("emoji_");
                    let template = server.config.as_ref().and_then(|c| c.server_emoji_remove_msg.as_ref());
                    self.log_fmt(server.log_channel, template, &map)?;
                },
                EmojisUpdateDiff::NameChanged(id, from, to) => {
                    let mut map = HashMap::new();
                    map.insert("emoji_id".to_string(), id.to_string());
                    map.insert("from".to_string(), from);
                    map.insert("to".to_string(), to);
                    let template = server.config.as_ref().and_then(|c| c.server_emoji_name_change_msg.as_ref());
                    self.log_fmt(server.log_channel, template, &map)?;
                }
            }
        }
        Ok(())
    }
}