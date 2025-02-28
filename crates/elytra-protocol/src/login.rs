﻿use super::packet::*;
use serde_json::json;
use tokio::io::Result;
use uuid::Uuid;

pub struct LoginStartPacket {
    pub username: String,
}

impl Packet for LoginStartPacket {
    fn read_from_buffer(buffer: &mut MinecraftPacketBuffer) -> Result<Self> {
        let username = buffer.read_string()?;

        Ok(LoginStartPacket { username })
    }
}

pub struct LoginSuccessPacket {
    pub uuid: Uuid,
    pub username: String,
}

impl LoginSuccessPacket {
    pub fn new(username: String) -> Self {
        // Generate offline mode UUID (Version 3, using username)
        let uuid = Uuid::new_v3(
            &Uuid::NAMESPACE_DNS,
            format!("OfflinePlayer:{}", username).as_bytes(),
        );

        LoginSuccessPacket { uuid, username }
    }
}

impl Packet for LoginSuccessPacket {
    fn packet_id() -> i32 {
        0x02
    }

    fn read_from_buffer(buffer: &mut MinecraftPacketBuffer) -> Result<Self> {
        let uuid = buffer.read_uuid()?;
        let username = buffer.read_string()?;

        Ok(LoginSuccessPacket { uuid, username })
    }

    fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> Result<()> {
        buffer.write_varint(Self::packet_id());
        buffer.write_uuid(self.uuid);
        buffer.write_string(&self.username);
        Ok(())
    }
}

pub struct LoginDisconnectPacket {
    pub reason: String,
}

impl LoginDisconnectPacket {
    #[allow(dead_code)]
    pub(crate) fn new(txt: String) -> Self {
        LoginDisconnectPacket {
            reason: json!({
                "text": txt
            })
            .to_string(),
        }
    }
}

impl Packet for LoginDisconnectPacket {
    fn read_from_buffer(buffer: &mut MinecraftPacketBuffer) -> Result<Self> {
        let reason = buffer.read_string()?;
        Ok(LoginDisconnectPacket { reason })
    }

    fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> Result<()> {
        buffer.write_varint(Self::packet_id());
        buffer.write_string(&self.reason);
        Ok(())
    }
}
