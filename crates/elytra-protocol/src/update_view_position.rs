use crate::packet::{MinecraftPacketBuffer, Packet};

/// Updates the client's location. This is used to determine what chunks should remain loaded and if a chunk load should be ignored; chunks outside the view distance may be unloaded.
/// Sent whenever the player moves across a chunk border horizontally, and also (according to testing) for any integer change in the vertical axis, even if it doesn't go across a chunk section border.
pub struct UpdateViewPositionPacket {
    chunk_x: i32,
    chunk_z: i32,
}

impl UpdateViewPositionPacket {
    pub fn new(chunk_x: i32, chunk_z: i32) -> Self {
        UpdateViewPositionPacket { chunk_x, chunk_z }
    }
}

impl Packet for UpdateViewPositionPacket {
    fn packet_id() -> i32
    where
        Self: Sized,
    {
        0x40
    }

    fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> std::io::Result<()> {
        buffer.write_varint(Self::packet_id());
        buffer.write_varint(self.chunk_x);
        buffer.write_varint(self.chunk_z);

        Ok(())
    }
}
