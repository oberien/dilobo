mod message;
mod member;
mod role;
mod ban;
mod emoji;
mod channel;
mod pins;
mod reaction;

use bot::Bot;

use discord::Result;
use discord::model::{Event, PossibleServer};

impl Bot {
    pub fn handle_event(&mut self, evt: Event) -> Result<()> {
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
            // TODO: Analyse PresenceUpdates as they contain user changes
            // Event::PresenceUpdate
            // Event::PresencesReplace
            // Event::RelationshipAdd
            // Event::RelationshipRemove
            Event::MessageCreate(msg) => {
                self.handle_message_create(msg)?;
            },
            Event::MessageUpdate(update) => {
                self.handle_message_update(update)?;
            },
            // Event::MessageAck
            Event::MessageDelete(del) => {
                self.handle_message_delete(del)?;
            },
            Event::MessageDeleteBulk(del) => {
                self.handle_message_delete_bulk(del)?;
            },
            Event::ServerCreate(server) => {
                match server {
                    PossibleServer::Online(server) => self.add_server(server),
                    _ => {}
                }
            },
            // TODO: update server
            // Event::ServerUpdate
            // TODO: handle offline servers
            // Event::ServerDelete
            Event::ServerMemberAdd(server_id, member) => {
                self.handle_server_member_add(server_id, member)?;
            },
            Event::ServerMemberUpdate(update) => {
                self.handle_server_member_update(update)?;
            },
            Event::ServerMemberRemove(server_id, user) => {
                self.handle_server_member_remove(server_id, user)?;
            },
            // Event::ServerMembersChunk
            // Event::ServerSync
            Event::ServerRoleCreate(server_id, role) => {
                self.handle_server_role_create(server_id, role)?;
            },
            Event::ServerRoleUpdate(server_id, role) => {
                self.handle_server_role_update(server_id, role)?;
            },
            Event::ServerRoleDelete(server_id, role_id) => {
                self.handle_server_role_delete(server_id, role_id)?;
            },
            Event::ServerBanAdd(server_id, user) => {
                self.handle_server_ban_add(server_id, user)?;
            },
            Event::ServerBanRemove(server_id, user) => {
                self.handle_server_ban_remove(server_id, user)?;
            },
            // Event:ServerIntegrationsUpdate
            Event::ServerEmojisUpdate(server_id, emojis) => {
                self.handle_server_emojis_update(server_id, emojis)?;
            },
            Event::ChannelCreate(channel) => {
                self.handle_channel_create(channel)?;
            },
            Event::ChannelUpdate(channel) => {
                self.handle_channel_update(channel)?;
            },
            Event::ChannelDelete(channel) => {
                self.handle_channel_delete(channel)?;
            },
            Event::ChannelPinsAck(ack) => {
                self.handle_channel_pins_ack(ack)?;
            },
            Event::ChannelPinsUpdate(update) => {
                self.handle_channel_pins_update(update)?;
            },
            Event::ReactionAdd(reaction) => {
                self.handle_reaction_add(reaction)?;
            }
            Event::ReactionRemove(reaction) => {
                self.handle_reaction_remove(reaction)?;
            },
            _ => ()
        }
        Ok(())
    }
}