use std::time::Duration;

use discord::model::{ServerInfo, ChannelId, MessageId, Message};

use expiring_map::ExpiringMap;

pub struct Server {
    pub server: ServerInfo,
    pub log_channel: ChannelId,
    pub messages: ExpiringMap<MessageId, Message>,
}

impl Server {
    pub fn new(server: ServerInfo, log_channel: ChannelId) -> Server {
        Server {
            server: server,
            log_channel: log_channel,
            messages: ExpiringMap::new(Duration::from_secs(300)),
        }
    }
}
