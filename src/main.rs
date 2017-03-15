extern crate toml;
extern crate rustc_serialize;
extern crate discord;
extern crate strfmt;
extern crate serde_json;

mod expiring_map;
mod modelext;
mod config;
mod bot;

use std::io::{self, Write};

use config::{Config, BotConfig};
use bot::Bot;

fn main() {
    println!("Reading config...");
    let mut config = Config::load("Config.toml");
    println!("Config read successfully");
    if let None = config.bot {
        print!("Please insert the bot token: ");
        io::stdout().flush().expect("could not flush stdout");
        let stdin = io::stdin();
        let mut token = String::new();
        stdin.read_line(&mut token).expect("could not read from stdin");
        let token = token.trim().to_string();
        config.bot = Some(BotConfig { token: token });
        config.save("Config.toml");
    }
    loop {
        let mut bot = Bot::new(config.clone());
        if let Err(err) = bot.run() {
            println!("Error during bot.run: {:?}", err);
        }
    }
}
