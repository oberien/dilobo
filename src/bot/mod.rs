mod server;
mod handle;

use std::collections::HashMap;

use strfmt::strfmt;
use discord::{Discord, Connection};
use discord::model::{
    CurrentUser,
    OnlineStatus,
    ServerId,
    ChannelId,
    LiveServer,
    PossibleServer,
};

use errors::*;
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
    pub fn new(config: Config) -> Result<Bot> {
        let discord = Discord::from_bot_token(&unwrap!(config.bot.as_ref(), err ConfigError, "No bot token").token)?;
        let (con, mut ready) = discord.connect()?;
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
                PossibleServer::Online(server) => bot.add_server(server)?,
                _ => {}
            }
        }
        Ok(bot)
    }

    fn add_server(&mut self, server: LiveServer) -> Result<()> {
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
                    // This case has already been checked with config.validate() in main()
                    None => unreachable!()
                }
            }
        });
        // not in server config
        if let None = index {
            let server = Server::new(server, None, None);
            let server_id = server.id;
            self.servers.insert(server.id, server);
            return Err(ErrorKind::ServerConfigError(server_id, "No configuration found".to_string()).into());
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
            return Ok(());
        }
        let server = Server::new(server, Some(server_config), log_channel);
        if let None = log_channel {
            println!("Added Server but couldn't find log channel {:?}", server.name);
        } else {
            println!("Successfully logging for server {:?}", server.name);
            self.log(server.log_channel, "Bot started successfully and is logging to this channel.")?;
        }
        println!();
        self.servers.insert(server.id, server);
        Ok(())
    }

    // returns if event was handled
    // Otherwise the server needs to be restarted
    fn handle_err(&mut self, err: Error) -> bool {
        match err.kind() {
            &ErrorKind::ServerConfigError(server, ref msg) => {
                let channel = self.server_by_server(server).unwrap().log_channel;
                let _ = self.log(channel, msg);
                return true;
            },
            &ErrorKind::FormatError(server, ref err) => {
                let channel = self.server_by_server(server).unwrap().log_channel;
                let _ = self.log(channel, &err.to_string());
                return true;
            }
            _ => ()
        }
        let mut errmsg = String::new();

        use ::std::fmt::Write as FmtWrite;
        use ::std::io::Write as IoWrite;
        writeln!(&mut errmsg, "error: {}", err).unwrap();

        for e in err.iter().skip(1) {
            writeln!(&mut errmsg, "caused by: {}", e).unwrap();
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        let stderr = &mut ::std::io::stderr();
        writeln!(stderr, "Uncaught error: {}", errmsg)
            .expect("failed writing to stderr");
        if let Some(backtrace) = err.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace)
                .expect("failed writing to stderr");
        }

        for server in self.servers.values() {
            // ignore errors, we are restarting anyways
            let _ = self.log(server.log_channel, &format!("Got uncaught error and need to restart.\
                    Please report this to oberien#2194 or on github ( \
                    https://github.com/oberien/dilobo/issues/new ):\n {}", errmsg));
        }
        return false;
    }

    pub fn run(&mut self) {
        loop {
            let evt = match self.con.recv_event() {
                Ok(evt) => evt,
                Err(err) => {
                    if !self.handle_err(err.into()) {
                        return;
                    }
                    continue;
                }
            };
            println!("evt: {:?}", evt);
            match self.handle_event(evt) {
                Ok(_) => (),
                Err(err) => {
                    if !self.handle_err(err) {
                        return;
                    }
                }
            }
            println!();
        }
    }

    fn server_by_channel(&self, channel_id: ChannelId) -> Result<&Server> {
        let server_id = unwrap!(self.channels.get(&channel_id),
                "could not find server for channel {}", channel_id);
        Ok(unwrap!(self.servers.get(&server_id),
            "could not find server for server_id {}", server_id))
    }
    fn server_by_server(&self, server_id: ServerId) -> Result<&Server> {
        Ok(unwrap!(self.servers.get(&server_id),
            "could not find server for server_id {}", server_id))
    }
    fn server_by_channel_mut(&mut self, channel_id: ChannelId) -> Result<&mut Server> {
        let server_id = unwrap!(self.channels.get(&channel_id),
            "could not find server for channel {}", channel_id);
        Ok(unwrap!(self.servers.get_mut(&server_id),
            "could not find server for server_id {}", server_id))
    }
    fn server_by_server_mut(&mut self, server_id: ServerId) -> Result<&mut Server> {
        Ok(unwrap!(self.servers.get_mut(&server_id),
            "could not find server for server_id {}", server_id))
    }

    fn log_fmt(&self, log_channel: Option<ChannelId>, fmt: Option<&String>, map: &HashMap<String, String>) -> Result<()> {
        if let Some(fmt) = fmt {
            let msg = match (strfmt(&fmt, map), log_channel) {
                (Ok(msg), _) => msg,
                (Err(err), Some(channel)) => return Err(ErrorKind::FormatError(self.server_by_channel(channel)?.id, err).into()),
                (Err(_), None) => return Err(ErrorKind::ConfigError("No log channel found".to_string()).into())
            };
            self.log(log_channel, &msg)?;
        }
        Ok(())
    }

    fn log(&self, log_channel: Option<ChannelId>, msg: &str) -> Result<()> {
        let log_channel = unwrap!(log_channel, err ConfigError, "No log channel found");
        self.dis.send_message(log_channel, msg, "", false).map(|_| ())?;
        Ok(())
    }
}
