use std::time::Duration;

use discord::model::{ServerInfo, ChannelId, MessageId, Message};

use expiring_map::ExpiringMap;
use config::ServerConfig;

#[derive(Debug, Clone)]
pub struct Server {
    pub config: ServerConfig,
    pub server: ServerInfo,
    pub log_channel: ChannelId,
    pub messages: ExpiringMap<MessageId, Message>,
}

impl Server {
    pub fn new(config: ServerConfig, server: ServerInfo, log_channel: ChannelId) -> Server {
        Server {
            config: config,
            server: server,
            log_channel: log_channel,
            messages: ExpiringMap::new(Duration::from_secs(300)),
        }
    }
}
