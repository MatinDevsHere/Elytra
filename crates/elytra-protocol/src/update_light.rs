use crate::packet::{MinecraftPacketBuffer, Packet};
use std::io;

/// Update Light packet
/// Updates light levels for a chunk.
pub struct UpdateLightPacket {
    chunk_x: i32,
    chunk_z: i32,
    trust_edges: bool,
    sky_light_mask: i32,
    block_light_mask: i32,
    empty_sky_light_mask: i32,
    empty_block_light_mask: i32,
    sky_light_arrays: Vec<Vec<u8>>,
    block_light_arrays: Vec<Vec<u8>>,
}

impl UpdateLightPacket {
    pub fn new(
        chunk_x: i32,
        chunk_z: i32,
        trust_edges: bool,
        sky_light_mask: i32,
        block_light_mask: i32,
        empty_sky_light_mask: i32,
        empty_block_light_mask: i32,
        sky_light_arrays: Vec<Vec<u8>>,
        block_light_arrays: Vec<Vec<u8>>,
    ) -> io::Result<Self> {
        // Validate all arrays are exactly 2048 bytes
        for array in &sky_light_arrays {
            if array.len() != 2048 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Sky light array must be exactly 2048 bytes, got {}",
                        array.len()
                    ),
                ));
            }
        }
        for array in &block_light_arrays {
            if array.len() != 2048 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Block light array must be exactly 2048 bytes, got {}",
                        array.len()
                    ),
                ));
            }
        }

        Ok(Self {
            chunk_x,
            chunk_z,
            trust_edges,
            sky_light_mask,
            block_light_mask,
            empty_sky_light_mask,
            empty_block_light_mask,
            sky_light_arrays,
            block_light_arrays,
        })
    }
}

impl Packet for UpdateLightPacket {
    fn packet_id() -> i32 {
        0x23
    }

    //noinspection GrazieInspection
    fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> io::Result<()> {
        buffer.write_varint(Self::packet_id());

        // Write chunk coordinates
        buffer.write_varint(self.chunk_x);
        buffer.write_varint(self.chunk_z);

        // Write trust edges flag
        buffer.write_bool(self.trust_edges);

        // Write light masks
        buffer.write_varint(self.sky_light_mask);
        buffer.write_varint(self.block_light_mask);
        buffer.write_varint(self.empty_sky_light_mask);
        buffer.write_varint(self.empty_block_light_mask);

        // Write sky light arrays
        for array in &self.sky_light_arrays {
            assert_eq!(
                array.len(),
                2048,
                "Sky light array must be exactly 2048 bytes"
            );
            buffer.write_varint(2048);
            buffer.buffer.extend_from_slice(array);
        }

        // Write block light arrays
        for array in &self.block_light_arrays {
            assert_eq!(
                array.len(),
                2048,
                "Block light array must be exactly 2048 bytes"
            );
            buffer.write_varint(2048);
            buffer.buffer.extend_from_slice(array);
        }

        Ok(())
    }
}
