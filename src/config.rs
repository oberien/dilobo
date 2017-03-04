use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use toml::{self, Parser, Decoder, Value};
use rustc_serialize::Decodable;

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
}
