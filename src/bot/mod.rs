mod server;

use std::collections::HashMap;

use strfmt::strfmt;
use discord::Result;
use discord::{Discord, Connection};
use discord::model::{
    CurrentUser,
    OnlineStatus,
    Event,
    ServerId,
    ChannelId,
    Channel,
    MessageDelete,
};

use config::Config;
use modelext::MergeIntoMap;
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

    fn handle_event(&mut self, evt: Event) -> Result<()> {
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
                {
                    let mut server = self.server_by_channel_mut(msg.channel_id);
                    server.messages.insert(msg.id, msg.clone());
                }
                let server = self.server_by_channel(msg.channel_id);
                // ignore new messages in log channel which we have created
                if msg.channel_id != server.log_channel || msg.author.id != self.user.id {
                    let map = msg.into_map();
                    self.log_fmt(&server, server.config.message_create_msg.as_ref(), &map)?;
                }
            },
            Event::MessageUpdate(update) => {
                let server = self.server_by_channel(update.channel_id);
                // ignore log channel
                // TODO: only ignore if it's a media embed update
                if server.log_channel != update.channel_id {
                    self.log(&server, &format!("Message Updated: {:?}", update))?;
                }
            },
            // Event::MessageAck
            Event::MessageDelete(del) => {
                let message;
                {
                    let server = self.server_by_channel_mut(del.channel_id);
                    message = server.messages.remove(&del.message_id);
                }
                let server = self.server_by_channel(del.channel_id);
                if let Some(msg) = message {
                    let map = msg.clone().into_map();
                    self.log_fmt(&server, server.config.message_delete_cached_msg.as_ref(), &map)?;
                } else {
                    let map = del.into_map();
                    self.log_fmt(&server, server.config.message_delete_uncached_msg.as_ref(), &map)?;
                }
            },
            Event::MessageDeleteBulk(del) => {
                {
                    let server = self.server_by_channel(del.channel_id);
                    let mut map = HashMap::new();
                    map.insert("channel_id".to_string(), del.channel_id.to_string());
                    map.insert("count".to_string(), del.ids.len().to_string());
                    self.log_fmt(&server, server.config.message_delete_bulk_msg.as_ref(), &map)?;
                }
                let mut text = String::new();
                for message_id in del.ids {
                    let cached;
                    {
                        let server = self.server_by_channel_mut(del.channel_id);
                        cached = server.messages.remove(&message_id);
                    }
                    let server = self.server_by_channel(del.channel_id);
                    let line = match cached {
                        // TODO: use error_chain instead of unwrap
                        Some(msg) => server.config.message_delete_cached_msg
                            .as_ref().map(|fmt| strfmt(fmt, &msg.into_map()).unwrap()),
                        None => {
                            let del = MessageDelete {
                                channel_id: del.channel_id,
                                message_id: message_id,
                            };
                            server.config.message_delete_uncached_msg
                                .as_ref().map(|fmt| strfmt(fmt, &del.into_map()).unwrap())
                        }
                    };
                    if let Some(line) = line {
                        if text.len() + line.len() > 2000 {
                            let server = self.server_by_channel(del.channel_id);
                            self.log(&server, &text)?;
                            text = line;
                        } else {
                            text += "\n";
                            text += &line;
                        }
                    }
                }
                let server = self.server_by_channel(del.channel_id);
                self.log(&server, &text)?;
            },
            // Event::ServerCreate
            // Event::ServerUpdate
            // Event::ServerDelete
            Event::ServerMemberAdd(server_id, member) => {
                let server = self.server_by_server(server_id);
                let map = member.into_map();
                self.log_fmt(&server, server.config.server_member_add_msg.as_ref(), &map)?;
            },
            Event::ServerMemberUpdate(update) => {
                let server = self.server_by_server(update.server_id);
                self.log(&server, &format!("Member Changed: {:?}", update))?;
            },
            Event::ServerMemberRemove(server_id, user) => {
                let server = self.server_by_server(server_id);
                let map = user.into_map();
                self.log_fmt(&server, server.config.server_member_remove_msg.as_ref(), &map)?;
            },
            // Event::ServerMembersChunk
            // Event::ServerSync
            Event::ServerRoleCreate(server_id, role) => {
                let server = self.server_by_server(server_id);
                let map = role.into_map();
                self.log_fmt(&server, server.config.server_role_create_msg.as_ref(), &map)?;
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
                let map = user.into_map();
                self.log_fmt(&server, server.config.server_ban_add_msg.as_ref(), &map)?;
            },
            Event::ServerBanRemove(server_id, user) => {
                let server = self.server_by_server(server_id);
                let map = user.into_map();
                self.log_fmt(&server, server.config.server_ban_remove_msg.as_ref(), &map)?;
            },
            // Event:ServerIntegrationsUpdate
            Event::ServerEmojisUpdate(server_id, emojis) => {
                let server = self.server_by_server(server_id);
                self.log(&server, &format!("Emojis Changed: {:?}", emojis))?;
            },
            Event::ChannelCreate(channel) => {
                if let Channel::Public(channel) = channel {
                    self.channels.insert(channel.id, channel.server_id);
                    let server = self.server_by_channel(channel.id);
                    let map = channel.into_map();
                    self.log_fmt(&server, server.config.channel_create_msg.as_ref(), &map)?;
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
                    let channel_id = channel.id;
                    {
                        let server = self.server_by_channel(channel.id);
                        let map = channel.into_map();
                        self.log_fmt(&server, server.config.channel_delete_msg.as_ref(), &map)?;
                    }
                    self.channels.remove(&channel_id);
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
            Event::ReactionAdd(reaction) => {
                let server = self.server_by_channel(reaction.channel_id);
                let message = server.messages.get(&reaction.message_id);
                let mut map = reaction.into_map();
                // TODO: improve infos in map with channel and user
                if let Some(msg) = message {
                    msg.clone().merge_into_map_prefix(&mut map, "message_");
                    self.log_fmt(&server, server.config.reaction_add_cached_msg.as_ref(), &map)?;
                } else {
                    self.log_fmt(&server, server.config.reaction_add_uncached_msg.as_ref(), &map)?;
                }
            }
            Event::ReactionRemove(reaction) => {
                let server = self.server_by_channel(reaction.channel_id);
                let message = server.messages.get(&reaction.message_id);
                let mut map = reaction.into_map();
                // TODO: improve infos in map with channel and user
                if let Some(msg) = message {
                    msg.clone().merge_into_map_prefix(&mut map, "message_");
                    self.log_fmt(&server, server.config.reaction_remove_cached_msg.as_ref(), &map)?;
                } else {
                    self.log_fmt(&server, server.config.reaction_remove_uncached_msg.as_ref(), &map)?;
                }
            },
            _ => ()
        }
        Ok(())
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

    fn log_fmt(&self, server: &Server, fmt: Option<&String>, map: &HashMap<String, String>) -> Result<()> {
        if let Some(fmt) = fmt {
            // TODO: user error_chain instead of unwrap
            let msg = strfmt(&fmt, map).unwrap();
            self.log(server, &msg)?;
        }
        Ok(())
    }

    fn log(&self, server: &Server, msg: &str) -> Result<()> {
        self.dis.send_message(server.log_channel, msg, "", false).map(|_| ())
    }
}
