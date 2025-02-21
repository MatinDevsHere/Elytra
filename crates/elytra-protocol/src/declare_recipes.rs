use crate::packet::{MinecraftPacketBuffer, Packet};

pub struct DeclareRecipesPacket {
    recipes_count: u8, // Using u8 here because Elytra is going to send 0 anyway
}

impl Packet for DeclareRecipesPacket {
    fn packet_id() -> i32
    where
        Self: Sized,
    {
        0x5A
    }

    fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> std::io::Result<()> {
        buffer.write_varint(Self::packet_id());
        buffer.write_varint(self.recipes_count as i32);

        Ok(())
    }
}

impl DeclareRecipesPacket {
    pub fn new() -> Self {
        Self { recipes_count: 0 }
    }
}
