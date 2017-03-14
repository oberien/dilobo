mod server;
mod handle;

use std::collections::HashMap;

use strfmt::strfmt;
use discord::Result;
use discord::{Discord, Connection};
use discord::model::{
    CurrentUser,
    OnlineStatus,
    ServerId,
    ChannelId,
};

use config::Config;
use self::server::Server;

pub struct Bot {
    dis: Discord,
    con: Connection,
    user: CurrentUser,
    servers: HashMap<ServerId, Server>,
    channels: HashMap<ChannelId, ServerId>,
}

impl Bot {
    pub fn new(config: &Config) -> Bot {
        let discord = Discord::from_bot_token(&config.bot.as_ref().unwrap().token).unwrap();
        let (con, ready) = discord.connect().unwrap();
        println!("Logged in as {:?}", ready);
        println!();
        Bot {
            dis: discord,
            con: con,
            user: ready.user,
            servers: HashMap::new(),
            channels: HashMap::new(),
        }
    }

    pub fn init(&mut self, mut config: Config) -> Result<()> {
        self.con.set_presence(None, OnlineStatus::Online, false);
        let mut servers = self.dis.get_servers().unwrap();
        for server in servers.drain(..) {
            // check if server is configured
            let index = config.server.iter().enumerate().find(|&(_, ref s)| {
                match s.server_id {
                    Some(id) => match s.server_name {
                        Some(ref name) => server.id == ServerId(id) && server.name == *name,
                        None => server.id == ServerId(id)
                    },
                    None => match s.server_name {
                        Some(ref name) => server.name == *name,
                        None => panic!("No server_id or server_name given to identify the server.")
                    }
                }
            }).map(|(i, _)| i);
            // not in server config
            if let None = index {
                println!("The bot is member of unconfigured server: {:?}", server);
                continue;
            }
            let server_config = config.server.swap_remove(index.unwrap());
            let mut log_channel = None;

            let channels = self.dis.get_server_channels(server.id).unwrap();
            for channel in channels {
                self.channels.insert(channel.id, server.id);
                // check if this channel is the log channel
                let is_log_channel = match server_config.log_channel_id {
                    Some(id) => match server_config.log_channel_name {
                        Some(ref name) => channel.id == ChannelId(id) && channel.name == *name,
                        None => channel.id == ChannelId(id)
                    },
                    None => match server_config.log_channel_name {
                        Some(ref name) => channel.name == *name,
                        None => panic!("No log_channel_id or log_channel_name given to identify the channel.")
                    }
                };
                if is_log_channel {
                    log_channel = Some(channel.id);
                }
            }
            if log_channel == None {
                panic!("Couldn't find log channel for server.");
            }
            let log_channel = log_channel.unwrap();
            let server = Server::new(server_config, server, log_channel);
            println!("Successfully logging for server {:?}", server);
            println!();
            self.servers.insert(server.server.id, server);
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let evt = self.con.recv_event()?;
            println!("evt: {:?}", evt);
            self.handle_event(evt)?;
            println!();
            println!();
        }
    }

    fn server_by_channel(&self, channel_id: ChannelId) -> &Server {
        let server_id = self.channels.get(&channel_id)
            .expect(&format!("could not find server for channel {}", channel_id));
        self.servers.get(&server_id)
            .expect(&format!("could not find server for server_id {}", server_id))
    }
    fn server_by_server(&self, server_id: ServerId) -> &Server {
        self.servers.get(&server_id)
            .expect(&format!("could not find server for server_id {}", server_id))
    }
    fn server_by_channel_mut(&mut self, channel_id: ChannelId) -> &mut Server {
        let server_id = self.channels.get(&channel_id)
            .expect(&format!("could not find server for channel {}", channel_id));
        self.servers.get_mut(&server_id)
            .expect(&format!("could not find server for server_id {}", server_id))
    }
    fn server_by_server_mut(&mut self, server_id: ServerId) -> &mut Server {
        self.servers.get_mut(&server_id)
            .expect(&format!("could not find server for server_id {}", server_id))
    }

    fn log_fmt(&self, log_channel: ChannelId, fmt: Option<&String>, map: &HashMap<String, String>) -> Result<()> {
        if let Some(fmt) = fmt {
            // TODO: user error_chain instead of unwrap
            let msg = strfmt(&fmt, map).unwrap();
            self.log(log_channel, &msg)?;
        }
        Ok(())
    }

    fn log(&self, log_channel: ChannelId, msg: &str) -> Result<()> {
        self.dis.send_message(log_channel, msg, "", false).map(|_| ())
    }
}
