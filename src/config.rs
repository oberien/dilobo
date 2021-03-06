use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};

use toml::{self, Parser, Decoder, Value};
use rustc_serialize::Decodable;

use errors::*;

#[derive(Debug, RustcDecodable, RustcEncodable, Clone)]
pub struct Config {
    pub bot: Option<BotConfig>,
    pub server: Vec<ServerConfig>,
}

#[derive(Debug, RustcDecodable, RustcEncodable, Clone)]
pub struct BotConfig {
    pub token: String,
}

#[derive(Debug, RustcDecodable, RustcEncodable, Clone)]
pub struct ServerConfig {
    pub server_id: Option<u64>,
    pub server_name: Option<String>,
    pub log_channel_id: Option<u64>,
    pub log_channel_name: Option<String>,
    pub verbose: Option<bool>,
    // Format strings for events
    //pub ready_msg: Option<String>,
    //pub resumed_msg: Option<String>,
    //pub user_update_msg: Option<String>,
    //pub user_note_update_msg: Option<String>,
    //pub user_settings_update_msg: Option<String>,
    //pub user_server_settings_update_msg: Option<String>,
    // TODO: Differenciate between updates
    //pub voice_state_update_msg: Option<String>,
    //pub call_create_msg: Option<String>,
    // TODO: Differenciate between updates
    //pub call_update_msg: Option<String>,
    //pub call_delete_msg: Option<String>,
    //pub channel_recipient_add_msg: Option<String>,
    //pub channel_recipient_remove_msg: Option<String>,
    //pub typing_start_msg: Option<String>,
    // TODO: Differenciate between updates
    //pub presence_update_msg: Option<String>,
    //pub presences_replace_msg: Option<String>,
    //pub relationship_add_msg: Option<String>,
    //pub relationship_remove_msg: Option<String>,
    pub message_create_msg: Option<String>,
    pub message_update_uncached_msg: Option<String>,
    pub message_update_kind_msg: Option<String>,
    pub message_update_content_msg: Option<String>,
    pub message_update_nonce_msg: Option<String>,
    pub message_update_tts_msg: Option<String>,
    pub message_update_pinned_msg: Option<String>,
    pub message_update_unpinned_msg: Option<String>,
    pub message_update_edited_time_msg: Option<String>,
    pub message_update_mention_everyone_msg: Option<String>,
    pub message_update_mention_added_msg: Option<String>,
    pub message_update_mention_removed_msg: Option<String>,
    pub message_update_mention_role_added_msg: Option<String>,
    pub message_update_mention_role_removed_msg: Option<String>,
    pub message_update_attachment_added_msg: Option<String>,
    pub message_update_attachment_removed_msg: Option<String>,
    pub message_update_embeds_added_msg: Option<String>,
    pub message_update_embeds_removed_msg: Option<String>,
    pub message_delete_cached_msg: Option<String>,
    pub message_delete_uncached_msg: Option<String>,
    pub message_delete_bulk_msg: Option<String>,
    // TODO: Differenciate between updates
    //pub server_update_msg: Option<String>,
    pub server_member_add_msg: Option<String>,
    pub server_member_role_add_msg: Option<String>,
    pub server_member_role_remove_msg: Option<String>,
    pub server_member_nick_change_msg: Option<String>,
    pub server_member_no_change_msg: Option<String>,
    pub server_member_remove_msg: Option<String>,
    //pub server_members_chunk_msg: Option<String>,
    //pub server_sync_msg: Option<String>,
    pub server_role_create_msg: Option<String>,
    // TODO: Differenciate between updates
    //pub server_role_update_msg: Option<String>,
    //pub server_role_delete_msg: Option<String>,
    pub server_ban_add_msg: Option<String>,
    pub server_ban_remove_msg: Option<String>,
    pub server_emoji_add_msg: Option<String>,
    pub server_emoji_remove_msg: Option<String>,
    pub server_emoji_name_change_msg: Option<String>,
    pub channel_create_msg: Option<String>,
    pub channel_update_no_change_msg: Option<String>,
    pub channel_update_name_msg: Option<String>,
    pub channel_update_user_perms_msg: Option<String>,
    pub channel_update_role_perms_msg: Option<String>,
    pub channel_update_topic_msg: Option<String>,
    pub channel_update_position_msg: Option<String>,
    pub channel_update_bitrate_msg: Option<String>,
    pub channel_update_user_limit_msg: Option<String>,
    pub channel_delete_msg: Option<String>,
    //pub channel_pins_ack_msg: Option<String>,
    // TODO: Differenciate between updates
    //pub channel_pins_update_msg: Option<String>,
    pub reaction_add_cached_msg: Option<String>,
    pub reaction_add_uncached_msg: Option<String>,
    pub reaction_remove_cached_msg: Option<String>,
    pub reaction_remove_uncached_msg: Option<String>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Config {
        let mut config = String::new();
        {
            let mut config_file = File::open(path).expect("Failed to open config file");
            config_file.read_to_string(&mut config).expect("Failed to read config file");
        }

        let mut parser = Parser::new(&config);

        let parsed = match parser.parse() {
            Some(x) => x,
            None => {
                for e in parser.errors {
                    println!("{}", e);
                }
                panic!("Failed to parse config");
            }
        };

        match Decodable::decode(&mut Decoder::new(Value::Table(parsed))) {
            Ok(x) => x,
            Err(e) => panic!("Failed to decode config: {}", e),
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) {
            let contents = toml::encode_str(self);
            let mut file = OpenOptions::new().create(true).write(true)
                .truncate(true).open(path).expect("Failed to open config file");
            write!(file, "{}", contents).expect("Failed to write config file");
    }

    pub fn validate(&mut self) -> Result<()> {
        if let None = self.bot {
            print!("Please insert the bot token: ");
            io::stdout().flush().expect("could not flush stdout");
            let stdin = io::stdin();
            let mut token = String::new();
            stdin.read_line(&mut token).expect("could not read from stdin");
            let token = token.trim().to_string();
            self.bot = Some(BotConfig { token: token });
            self.save("Config.toml");
        }
        for server in self.server.iter() {
            if server.server_name.is_none() && server.server_id.is_none() {
                return Err(ErrorKind::ConfigError("No server_id or server_name given".to_string()).into())
            }
        }
        Ok(())
    }
}
