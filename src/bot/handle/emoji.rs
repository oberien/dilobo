use bot::Bot;

use discord::Result;
use discord::model::{
    ServerId,
    Emoji,
};

impl Bot {
    pub fn handle_server_emojis_update(&self, server_id: ServerId, emojis: Vec<Emoji>) -> Result<()> {
        // TODO: implement function
        let server = self.server_by_server(server_id);
        self.log(server.log_channel, &format!("Emojis Changed: {:?}", emojis))?;
        Ok(())
    }
}