use crate::protocol::packet::{MinecraftPacketBuffer, Packet};

pub struct HeldItemChangePacket {
    slot: u8,
}

impl Packet for HeldItemChangePacket {
    fn packet_id() -> i32
    where
        Self: Sized,
    {
        0x3F
    }

    fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> std::io::Result<()> {
        buffer.write_varint(Self::packet_id());
        buffer.write_u8(self.slot);

        Ok(())
    }
}

impl HeldItemChangePacket {
    pub fn new(slot: u8) -> HeldItemChangePacket {
        HeldItemChangePacket { slot }
    }
}
