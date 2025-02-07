use super::packet::*;
use crate::protocol::packet::{MinecraftPacketBuffer, Packet};
use serde_json::json;
use tokio::io::*;

pub struct StatusRequestPacket;

impl Packet for StatusRequestPacket {
    fn read(_buffer: &mut MinecraftPacketBuffer) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        Ok(StatusRequestPacket)
    }
}

pub struct StatusResponsePacket {
    pub response_json: String,
}

impl StatusResponsePacket {
    pub fn new() -> Self {
        let status_json = json!({
            "version": {
                "name": "1.16.5",
                "protocol": 754
            },
            "players": {
                "max": 100,
                // TODO: Online players should be fetched dynamically
                "online": 0,
                "sample": []
            },
            "description": {
                "text": "An Elytra Server"
            }
        });

        StatusResponsePacket {
            response_json: status_json.to_string(),
        }
    }
}

impl Packet for StatusResponsePacket {
    fn write(&self, buffer: &mut MinecraftPacketBuffer) -> Result<()> {
        buffer.write_varint(Self::packet_id());
        buffer.write_string(&self.response_json);
        Ok(())
    }
}
