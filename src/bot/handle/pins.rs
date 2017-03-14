use bot::Bot;

use discord::Result;
use discord::model::{
    ChannelPinsAck,
    ChannelPinsUpdate,
};

impl Bot {
    pub fn handle_channel_pins_ack(&self, ack: ChannelPinsAck) -> Result<()> {
        // TODO: implement function
        let server = self.server_by_channel(ack.channel_id);
        self.log(&server, &format!("Pins Ack: {:?}", ack))?;
        Ok(())
    }

    pub fn handle_channel_pins_update(&self, update: ChannelPinsUpdate) -> Result<()> {
        // TODO: implement function
        let server = self.server_by_channel(update.channel_id);
        self.log(&server, &format!("Pins Update: {:?}", update))?;
        Ok(())
    }
}
