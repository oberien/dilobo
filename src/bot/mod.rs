mod server;

use std::collections::HashMap;

use discord::Result;
use discord::{Discord, Connection};
use discord::model::{OnlineStatus, Event, ServerId, ChannelId, Channel};

use config::Config;
use self::server::Server;

pub struct Bot {
    dis: Discord,
    con: Connection,
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
            servers: HashMap::new(),
            channels: HashMap::new(),
        }
    }

    pub fn init(&mut self, config: &Config) -> Result<()> {
        self.con.set_presence(None, OnlineStatus::Online, false);
        let mut servers = self.dis.get_servers().unwrap();
        for server in servers.drain(..) {
            // check if server is configured
            let server_config = config.server.iter().find(|s| {
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
            if let None = server_config {
                println!("The bot is member of unconfigured server: {:?}", server);
                continue;
            }
            let server_config = server_config.unwrap();
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
            let server = Server::new(server, log_channel);
            self.servers.insert(server.server.id, server);
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let evt = self.con.recv_event()?;
            println!("evt: {:?}", evt);
            match evt {
                // Event::Ready
                // Event::Resumed
                // Event::UserUpdate
                // Event::UserNoteUpdate
                // Event::UserSettingsUpdate
                // Event::UserServerSettingsUpdate
                // Event::VoiceStateUpdate
                // Event::CallCreate
                // Event::CallUpdate
                // Event::CallDelete
                // Event::ChannelRecipientAdd
                // Event::ChannelRecipientRemove
                // Event::TypingStart
                // Event::PresenceUpdate
                // Event::PresencesReplace
                // Event::RelationshipAdd
                // Event::RelationshipRemove
                Event::MessageCreate(msg) => {
                    let server = self.server_by_channel_mut(msg.channel_id);
                    server.messages.insert(msg.id, msg);
                },
                Event::MessageUpdate(update) => {
                    let server = self.server_by_channel(update.channel_id);
                    // ignore log channel
                    // TODO: only ignore if it's a media embed update
                    if server.log_channel == update.channel_id {
                        continue;
                    }
                    self.log(&server, &format!("Message Updated: {:?}", update))?;
                },
                // Event::MessageAck
                Event::MessageDelete(del) => {
                    let server = self.server_by_channel(del.channel_id);
                    self.log(&server, &format!("Message Deleted: {:?}", del))?;
                },
                Event::MessageDeleteBulk(del) => {
                    let server = self.server_by_channel(del.channel_id);
                    self.log(&server, &format!("Message Bulk Delete: {:?}", del))?;
                },
                // Event::ServerCreate
                // Event::ServerUpdate
                // Event::ServerDelete
                Event::ServerMemberAdd(server_id, member) => {
                    let server = self.server_by_server(server_id);
                    self.log(&server, &format!("Member Joined: {:?}", member))?;
                },
                Event::ServerMemberUpdate(update) => {
                    let server = self.server_by_server(update.server_id);
                    self.log(&server, &format!("Member Changed: {:?}", update))?;
                },
                Event::ServerMemberRemove(server_id, user) => {
                    let server = self.server_by_server(server_id);
                    self.log(&server, &format!("Member Left: {:?}", user))?;
                },
                // Event::ServerMembersChunk
                // Event::ServerSync
                Event::ServerRoleCreate(server_id, role) => {
                    let server = self.server_by_server(server_id);
                    self.log(&server, &format!("Role Created: {:?}", role))?;
                },
                Event::ServerRoleUpdate(server_id, role) => {
                    let server = self.server_by_server(server_id);
                    self.log(&server, &format!("Role Changed: {:?}", role))?;
                },
                Event::ServerRoleDelete(server_id, role_id) => {
                    let server = self.server_by_server(server_id);
                    self.log(&server, &format!("Role Deleted: {:?}", role_id))?;
                },
                Event::ServerBanAdd(server_id, user) => {
                    let server = self.server_by_server(server_id);
                    self.log(&server, &format!("User Banned: {:?}", user))?;
                },
                Event::ServerBanRemove(server_id, user) => {
                    let server = self.server_by_server(server_id);
                    self.log(&server, &format!("User Unbanned: {:?}", user))?;
                },
                // Event:ServerIntegrationsUpdate
                Event::ServerEmojisUpdate(server_id, emojis) => {
                    let server = self.server_by_server(server_id);
                    self.log(&server, &format!("Emojis Changed: {:?}", emojis))?;
                },
                Event::ChannelCreate(channel) => {
                    if let Channel::Public(channel) = channel {
                        let server = self.server_by_channel(channel.id);
                        self.log(&server, &format!("Channel Created: {:?}", channel))?;
                    }
                },
                Event::ChannelUpdate(channel) => {
                    if let Channel::Public(channel) = channel {
                        let server = self.server_by_channel(channel.id);
                        self.log(&server, &format!("Channel Changed: {:?}", channel))?;
                    }
                },
                Event::ChannelDelete(channel) => {
                    if let Channel::Public(channel) = channel {
                        let server = self.server_by_channel(channel.id);
                        self.log(&server, &format!("Channel Deleted: {:?}", channel))?;
                    }
                },
                Event::ChannelPinsAck(ack) => {
                    let server = self.server_by_channel(ack.channel_id);
                    self.log(&server, &format!("Pins Ack: {:?}", ack))?;
                },
                Event::ChannelPinsUpdate(update) => {
                    let server = self.server_by_channel(update.channel_id);
                    self.log(&server, &format!("Pins Update: {:?}", update))?;
                },
                // Event::ReactionAdd
                Event::ReactionRemove(reaction) => {
                    let server = self.server_by_channel(reaction.channel_id);
                    self.log(&server, &format!("Reaction Removed: {:?}", reaction))?;
                },
                _ => ()
            }
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

    fn log(&self, server: &Server, msg: &str) -> Result<()> {
        self.dis.send_message(server.log_channel, msg, "", false).map(|_| ())
    }
}
