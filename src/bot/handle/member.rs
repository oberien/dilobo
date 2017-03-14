use bot::Bot;
use modelext::MergeIntoMap;

use discord::Result;
use discord::model::{
    ServerId,
    User,
    Member,
    ServerMemberUpdate,
};

impl Bot {
    pub fn handle_server_member_add(&self, server_id: ServerId, member: Member) -> Result<()> {
        // TODO: Add member to cache
        let server = self.server_by_server(server_id);
        let map = member.into_map();
        self.log_fmt(&server, server.config.server_member_add_msg.as_ref(), &map)?;
        Ok(())
    }

    pub fn handle_server_member_update(&self, update: ServerMemberUpdate) -> Result<()> {
        // TODO: calculate diff
        // TODO: update member in cache
        let server = self.server_by_server(update.server_id);
        self.log(&server, &format!("Member Changed: {:?}", update))?;
        Ok(())
    }

    pub fn handle_server_member_remove(&self, server_id: ServerId, user: User) -> Result<()> {
        // TODO: remove member from cache
        let server = self.server_by_server(server_id);
        let map = user.into_map();
        self.log_fmt(&server, server.config.server_member_remove_msg.as_ref(), &map)?;
        Ok(())
    }
}

