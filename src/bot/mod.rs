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
    LiveServer,
    PossibleServer,
};

use config::Config;
use self::server::Server;

pub struct Bot {
    config: Config,
    dis: Discord,
    con: Connection,
    user: CurrentUser,
    servers: HashMap<ServerId, Server>,
    channels: HashMap<ChannelId, ServerId>,
}

impl Bot {
    pub fn new(config: Config) -> Bot {
        let discord = Discord::from_bot_token(&config.bot.as_ref().unwrap().token).unwrap();
        let (con, mut ready) = discord.connect().unwrap();
        println!("Logged in as {:?}", ready.user.username);
        println!();
        let mut bot = Bot {
            config: config,
            dis: discord,
            con: con,
            user: ready.user,
            servers: HashMap::new(),
            channels: HashMap::new(),
        };
        bot.con.set_presence(None, OnlineStatus::Online, false);
        for server in ready.servers.drain(..) {
            match server {
                PossibleServer::Online(server) => bot.add_server(server),
                _ => {}
            }
        }
        bot
    }

    fn add_server(&mut self, server: LiveServer) {
        // regardless of if the server is configured or not, we need to have its full state
        // so we can keep it updated in case it is dynamically configured later
        for channel in server.channels.iter() {
            self.channels.insert(channel.id, server.id);
        }
        // check if server is configured
        let index = self.config.server.iter().position(|s| {
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
        });
        // not in server config
        if let None = index {
            println!("The bot is member of unconfigured server: {:?}", server.name);
            let server = Server::new(server, None, None);
            self.servers.insert(server.id, server);
            return;
        }
        let server_config = self.config.server.swap_remove(index.unwrap());
        let mut log_channel = None;
        let mut channel_missing = false;

        for channel in server.channels.iter() {
            // check if this channel is the log channel
            let is_log_channel = match server_config.log_channel_id {
                Some(id) => match server_config.log_channel_name {
                    Some(ref name) => channel.id == ChannelId(id) && channel.name == *name,
                    None => channel.id == ChannelId(id)
                },
                None => match server_config.log_channel_name {
                    Some(ref name) => channel.name == *name,
                    None => {
                        channel_missing = true;
                        break;
                    }
                }
            };
            if is_log_channel {
                log_channel = Some(channel.id);
                break;
            }
        }
        if channel_missing {
            println!("No log_channel_id or log_channel_name given to identify the channel.");
            let server = Server::new(server, Some(server_config), None);
            self.servers.insert(server.id, server);
            return;
        }
        let server = Server::new(server, Some(server_config), log_channel);
        if let None = log_channel {
            println!("Added Server but couldn't find log channel {:?}", server.name);
        } else {
            println!("Successfully logging for server {:?}", server.name);
        }
        println!();
        self.servers.insert(server.id, server);

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

    fn log_fmt(&self, log_channel: Option<ChannelId>, fmt: Option<&String>, map: &HashMap<String, String>) -> Result<()> {
        if let Some(fmt) = fmt {
            // TODO: user error_chain instead of unwrap
            let msg = strfmt(&fmt, map).unwrap();
            self.log(log_channel, &msg)?;
        }
        Ok(())
    }

    fn log(&self, log_channel: Option<ChannelId>, msg: &str) -> Result<()> {
        if let None = log_channel {
            // TODO: return proper error with error_chain
            return Ok(())
        }
        let log_channel = log_channel.unwrap();
        self.dis.send_message(log_channel, msg, "", false).map(|_| ())
    }
}
