use bot::Bot;
use modelext::MergeIntoMap;

use discord::Result;
use discord::model::{
    ServerId,
    Role,
    RoleId,
};

impl Bot {
    pub fn handle_server_role_create(&self, server_id: ServerId, role: Role) -> Result<()> {
        // TODO: add role to cache
        let server = self.server_by_server(server_id);
        let map = role.into_map();
        let template = server.config.as_ref().and_then(|c| c.server_role_create_msg.as_ref());
        self.log_fmt(server.log_channel, template, &map)?;
        Ok(())
    }

    pub fn handle_server_role_update(&self, server_id: ServerId, role: Role) -> Result<()> {
        // TODO: implement function
        // TODO: calculate diff
        // TODO: update role in cache
        let server = self.server_by_server(server_id);
        self.log(server.log_channel, &format!("Role Changed: {:?}", role))?;
        Ok(())
    }

    pub fn handle_server_role_delete(&self, server_id: ServerId, role_id: RoleId) -> Result<()> {
        // TODO: implement function
        // TODO: remove role in cache
        let server = self.server_by_server(server_id);
        self.log(server.log_channel, &format!("Role Deleted: {:?}", role_id))?;
        Ok(())
    }
}
