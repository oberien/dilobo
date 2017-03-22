// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]extern crate toml;

extern crate rustc_serialize;
extern crate discord;
extern crate strfmt;
extern crate serde_json;
#[macro_use]
extern crate error_chain;

#[macro_use]
mod errors;
mod expiring_map;
mod modelext;
mod config;
mod bot;

use std::time::{Instant, Duration};

use config::Config;
use bot::Bot;

fn main() {
    println!("Reading config...");
    let mut config = Config::load("Config.toml");
    println!("Config read successfully");
    config.validate().unwrap();

    let mintime = Duration::from_secs(5);
    let mut fastexits = 0;
    loop {
        let time = Instant::now();
        let mut bot = Bot::new(config.clone()).unwrap();
        bot.run();
        if time + mintime < Instant::now() {
            fastexits += 1;
        } else {
            fastexits = 0;
        }
        if fastexits > 3 {
            panic!("FUCK THIS SHIT I'M OUT");
        }
    }
}
