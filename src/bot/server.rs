use std::time::Duration;
use std::collections::HashMap;

use discord::model::{
    ServerId,
    LiveServer,
    ChannelId,
    RoleId,
    Role,
    MessageId,
    Message,
    UserId,
    Member,
    PublicChannel,
    VerificationLevel,
    EmojiId,
    Emoji,
};

use expiring_map::ExpiringMap;
use config::ServerConfig;

#[derive(Debug, Clone)]
pub struct Server {
    pub id: ServerId,
    pub name: String,
    pub owner_id: UserId,
    // voice_states
    pub roles: HashMap<RoleId, Role>,
    pub region: String,
    // presences
    pub members: HashMap<UserId, Member>,
    // joined_at
    pub icon: Option<String>,
    // large
    pub channels: HashMap<ChannelId, PublicChannel>,
    pub afk_timeout: u64,
    pub afk_channel_id: Option<ChannelId>,
    pub verification_level: VerificationLevel,
    pub emojis: HashMap<EmojiId, Emoji>,
    // features
    // splash
    // default_message_notifications
    // mfa_level
    pub log_channel: ChannelId,
    pub messages: ExpiringMap<MessageId, Message>,
    pub config: ServerConfig,
}

impl Server {
    pub fn new(config: ServerConfig, mut server: LiveServer, log_channel: ChannelId) -> Server {
        Server {
            id: server.id,
            name: server.name,
            owner_id: server.owner_id,
            roles: server.roles.drain(..).map(|role| (role.id, role)).collect(),
            region: server.region,
            members: server.members.drain(..).map(|member| (member.user.id, member)).collect(),
            icon: server.icon,
            channels: server.channels.drain(..).map(|channel| (channel.id, channel)).collect(),
            afk_timeout: server.afk_timeout,
            afk_channel_id: server.afk_channel_id,
            verification_level: server.verification_level,
            emojis: server.emojis.drain(..).map(|emoji| (emoji.id, emoji)).collect(),
            log_channel: log_channel,
            messages: ExpiringMap::new(Duration::from_secs(300)),
            config: config,
        }
    }
}
