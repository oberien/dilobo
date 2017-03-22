use bot::Bot;
use modelext::MergeIntoMap;

use discord::model::{
    ServerId,
    User,
};

use errors::*;

impl Bot {
    pub fn handle_server_ban_add(&self, server_id: ServerId, user: User) -> Result<()> {
        let server = self.server_by_server(server_id)?;
        let map = user.into_map()?;
        let template = server.config.as_ref().and_then(|c| c.server_ban_add_msg.as_ref());
        self.log_fmt(server.log_channel, template, &map)?;
        Ok(())
    }

    pub fn handle_server_ban_remove(&self, server_id: ServerId, user: User) -> Result<()> {
        let server = self.server_by_server(server_id)?;
        let map = user.into_map()?;
        let template = server.config.as_ref().and_then(|c| c.server_ban_remove_msg.as_ref());
        self.log_fmt(server.log_channel, template, &map)?;
        Ok(())
    }
}