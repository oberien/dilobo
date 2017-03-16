use bot::Bot;
use modelext::{MergeIntoMap, Diff, MemberUpdateDiff};

use discord::Result;
use discord::model::{
    ServerId,
    User,
    Member,
    ServerMemberUpdate,
};

impl Bot {
    pub fn handle_server_member_add(&mut self, server_id: ServerId, member: Member) -> Result<()> {
        {
            let server = self.server_by_server_mut(server_id);
            server.members.insert(member.user.id, member.clone());
        }
        let server = self.server_by_server(server_id);
        let map = member.into_map();
        self.log_fmt(server.log_channel, server.config.server_member_add_msg.as_ref(), &map)?;
        Ok(())
    }

    pub fn handle_server_member_update(&mut self, update: ServerMemberUpdate) -> Result<()> {
        let mut diffs;
        let member;
        {
            let server = self.server_by_server_mut(update.server_id);
            let member_ref = server.members.get_mut(&update.user.id).unwrap();
            member = member_ref.clone();
            diffs = member.diff(&update);
            for diff in diffs.iter() {
                diff.apply(member_ref);
            }
        }
        let server = self.server_by_server(update.server_id);
        if diffs.is_empty() {
            let map = member.into_map_prefix("member_");
            self.log_fmt(server.log_channel, server.config.server_member_no_change_msg.as_ref(), &map)?;
            return Ok(());
        }
        for diff in diffs.drain(..) {
            match diff {
                MemberUpdateDiff::RoleAdded(role_id) => {
                    let role = server.roles.get(&role_id).unwrap().clone();
                    let mut map = role.into_map_prefix("role_");
                    member.clone().merge_into_map_prefix(&mut map, "member_");
                    self.log_fmt(server.log_channel, server.config.server_member_role_add_msg.as_ref(), &map)?;
                },
                MemberUpdateDiff::RoleRemoved(role_id) => {
                    let role = server.roles.get(&role_id).unwrap().clone();
                    let mut map = role.into_map_prefix("role_");
                    member.clone().merge_into_map_prefix(&mut map, "member_");
                    self.log_fmt(server.log_channel, server.config.server_member_role_remove_msg.as_ref(), &map)?;
                },
                MemberUpdateDiff::NickChanged(from, to) => {
                    let mut map = member.clone().into_map_prefix("member_");
                    match from {
                        Some(s) => map.insert("from".to_string(), s),
                        None => map.insert("from".to_string(), "None".to_string())
                    };
                    match to {
                        Some(s) => map.insert("to".to_string(), s),
                        None => map.insert("to".to_string(), "None".to_string())
                    };
                    self.log_fmt(server.log_channel, server.config.server_member_nick_change_msg.as_ref(), &map)?;
                },
            }
        }
        Ok(())
    }

    pub fn handle_server_member_remove(&mut self, server_id: ServerId, user: User) -> Result<()> {
        {
            let mut server = self.server_by_server_mut(server_id);
            server.members.remove(&user.id);
        }
        let server = self.server_by_server(server_id);
        let map = user.into_map();
        self.log_fmt(server.log_channel, server.config.server_member_remove_msg.as_ref(), &map)?;
        Ok(())
    }
}

