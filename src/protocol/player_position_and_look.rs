use crate::protocol::packet::{MinecraftPacketBuffer, Packet};
use std::io;

/// Player Position And Look (clientbound)
/// Updates the player's position on the server. This packet will also close the "Downloading Terrain" screen when joining/respawning.
#[derive(Debug)]
pub struct PlayerPositionAndLook {
    /// Absolute or relative position, depending on Flags
    pub x: f64,
    /// Absolute or relative position, depending on Flags
    pub y: f64,
    /// Absolute or relative position, depending on Flags
    pub z: f64,
    /// Absolute or relative rotation on the X axis, in degrees
    pub yaw: f32,
    /// Absolute or relative rotation on the Y axis, in degrees
    pub pitch: f32,
    /// Bit field for relative/absolute positions and rotations
    pub flags: u8,
    /// Teleport ID for client confirmation
    pub teleport_id: i32,
}

impl Packet for PlayerPositionAndLook {
    fn packet_id() -> i32
    where
        Self: Sized,
    {
        0x34
    }

    fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> io::Result<()> {
        buffer.write_varint(Self::packet_id());

        // Write position coordinates as doubles
        buffer.write_f64(self.x)?;
        buffer.write_f64(self.y)?;
        buffer.write_f64(self.z)?;

        // Write rotation values as floats
        buffer.write_f32(self.yaw)?;
        buffer.write_f32(self.pitch)?;

        // Write flags as a byte
        buffer.write_u8(self.flags);

        // Write teleport ID as a VarInt
        buffer.write_varint(self.teleport_id);

        Ok(())
    }
}

impl PlayerPositionAndLook {
    pub fn new(x: f64, y: f64, z: f64, yaw: f32, pitch: f32, flags: u8, teleport_id: i32) -> Self {
        Self {
            x,
            y,
            z,
            yaw,
            pitch,
            flags,
            teleport_id,
        }
    }

    /// Flag constants for the flags field
    pub const RELATIVE_X: u8 = 0x01;
    pub const RELATIVE_Y: u8 = 0x02;
    pub const RELATIVE_Z: u8 = 0x04;
    pub const RELATIVE_Y_ROT: u8 = 0x08;
    pub const RELATIVE_X_ROT: u8 = 0x10;
}
