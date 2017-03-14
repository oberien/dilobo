use bot::Bot;
use modelext::MergeIntoMap;

use discord::Result;
use discord::model::{
    ServerId,
    User,
};

impl Bot {
    pub fn handle_server_ban_add(&self, server_id: ServerId, user: User) -> Result<()> {
        let server = self.server_by_server(server_id);
        let map = user.into_map();
        self.log_fmt(server.log_channel, server.config.server_ban_add_msg.as_ref(), &map)?;
        Ok(())
    }

    pub fn handle_server_ban_remove(&self, server_id: ServerId, user: User) -> Result<()> {
        let server = self.server_by_server(server_id);
        let map = user.into_map();
        self.log_fmt(server.log_channel, server.config.server_ban_remove_msg.as_ref(), &map)?;
        Ok(())
    }
}