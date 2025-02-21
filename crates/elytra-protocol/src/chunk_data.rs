use crate::packet::{MinecraftPacketBuffer, Packet};
use elytra_nbt::Tag;
use elytra_wotra::chunk::{BlockState, ChunkColumn};
use std::io::{self, Read};

// Add Read implementation for MinecraftPacketBuffer
impl Read for MinecraftPacketBuffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let remaining = self.buffer.len() - self.get_cursor();
        let to_read = buf.len().min(remaining);
        buf[..to_read]
            .copy_from_slice(&self.buffer[self.get_cursor()..self.get_cursor() + to_read]);
        self.advance_cursor(to_read);
        Ok(to_read)
    }
}

//noinspection GrazieInspection
/// Represents a chunk section in the chunk data packet
#[derive(Debug, Clone)]
pub struct ChunkSection {
    /// Number of non-air blocks in the section
    block_count: i16,
    /// Number of bits used per block
    bits_per_block: u8,
    /// Block states data array
    data_array: Vec<u64>,
    /// Block light values (4 bits per block)
    block_light: Vec<u8>,
    /// Sky light values (4 bits per block)
    sky_light: Option<Vec<u8>>,
    /// Palette for block states
    palette: Palette,
}

/// Represents the palette used for block states
#[derive(Debug, Clone)]
pub enum Palette {
    /// Direct palette using global block state IDs
    Direct,
    /// Indirect palette with a mapping of indices to block states
    Indirect {
        bits_per_block: u8,
        palette: Vec<u32>,
    },
}

/// Represents the chunk data packet
#[derive(Debug, Clone)]
pub struct ChunkDataPacket {
    /// X coordinate of the chunk
    pub chunk_x: i32,
    /// Z coordinate of the chunk
    pub chunk_z: i32,
    /// Whether this is a full chunk
    pub full_chunk: bool,
    /// Bitmask of chunk sections present in the packet
    pub primary_bit_mask: i32,
    /// Heightmap data as NBT
    pub heightmaps: Vec<Tag>,
    /// Biome data (only present if full_chunk is true)
    pub biomes: Option<Vec<i32>>,
    /// Chunk sections data
    pub sections: Vec<ChunkSection>,
    /// Block entities in NBT format
    pub block_entities: Vec<Tag>,
}

impl Packet for ChunkDataPacket {
    fn packet_id() -> i32 {
        0x22 // Chunk Data packet ID
    }

    fn read_from_buffer(buffer: &mut MinecraftPacketBuffer) -> io::Result<Self> {
        let packet_id = buffer.read_varint()?;
        if packet_id != Self::packet_id() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid packet ID: {}", packet_id),
            ));
        }

        let chunk_x = buffer.read_varint()?;
        let chunk_z = buffer.read_varint()?;
        let full_chunk = buffer.read_bool()?;
        let primary_bit_mask = buffer.read_varint()?;

        // Read heightmaps NBT
        let mut heightmaps = Vec::new();
        let (_, heightmap) = Tag::read(buffer)?;
        heightmaps.push(heightmap);

        // Read biomes if this is a full chunk
        let biomes = if full_chunk {
            let mut biome_data = Vec::with_capacity(1024);
            for _ in 0..1024 {
                biome_data.push(buffer.read_varint()?);
            }
            Some(biome_data)
        } else {
            None
        };

        // Read size of chunk data
        let _size = buffer.read_varint()?;

        // Read chunk sections
        let mut sections = Vec::new();
        for y in 0..16 {
            if (primary_bit_mask & (1 << y)) != 0 {
                // Read block count
                let block_count = buffer.read_u16()? as i16;

                // Read bits per block
                let bits_per_block = buffer.read_u8()?;

                // Create section with appropriate bits per block
                let mut section = ChunkSection::with_bits_per_block(bits_per_block);
                section.block_count = block_count;

                // Read palette
                if bits_per_block < 9 {
                    let palette_length = buffer.read_varint()?;
                    let mut palette_data = Vec::with_capacity(palette_length as usize);
                    for _ in 0..palette_length {
                        palette_data.push(buffer.read_varint()? as u32);
                    }
                    section.palette = Palette::Indirect {
                        bits_per_block,
                        palette: palette_data,
                    };
                } else {
                    section.palette = Palette::Direct;
                }

                // Read data array
                let data_array_length = buffer.read_varint()?;
                let mut data_array = Vec::with_capacity(data_array_length as usize);
                for _ in 0..data_array_length {
                    data_array.push(buffer.read_i64()? as u64);
                }
                section.data_array = data_array;

                // Read block light
                let mut block_light = vec![0; 2048];
                for i in 0..2048 {
                    block_light[i] = buffer.read_u8()?;
                }
                section.block_light = block_light;

                // Read sky light if present
                // TODO: Check dimension type properly
                let sky_light = if true {
                    let mut sky_light_data = vec![0; 2048];
                    for i in 0..2048 {
                        sky_light_data[i] = buffer.read_u8()?;
                    }
                    Some(sky_light_data)
                } else {
                    None
                };
                section.sky_light = sky_light;

                sections.push(section);
            }
        }

        // Read block entities
        let block_entity_count = buffer.read_varint()?;
        let mut block_entities = Vec::with_capacity(block_entity_count as usize);
        for _ in 0..block_entity_count {
            let (_, tag) = Tag::read(buffer)?;
            block_entities.push(tag);
        }

        Ok(ChunkDataPacket {
            chunk_x,
            chunk_z,
            full_chunk,
            primary_bit_mask,
            heightmaps,
            biomes,
            sections,
            block_entities,
        })
    }

    fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> io::Result<()> {
        buffer.write_varint(Self::packet_id());

        buffer.write_varint(self.chunk_x);
        buffer.write_varint(self.chunk_z);
        buffer.write_bool(self.full_chunk);
        buffer.write_varint(self.primary_bit_mask);

        // Write heightmaps NBT
        for heightmap in &self.heightmaps {
            heightmap.write(buffer, "")?;
        }

        // Write biomes if this is a full chunk
        if self.full_chunk {
            if let Some(biomes) = &self.biomes {
                for biome in biomes {
                    buffer.write_varint(*biome);
                }
            }
        }

        // Create a temporary buffer for the chunk data
        let mut temp_buffer = MinecraftPacketBuffer::new();

        // Write sections
        for section in &self.sections {
            // Block count
            temp_buffer.write_u16(section.block_count as u16);
            // Bits per block
            temp_buffer.write_u8(section.bits_per_block);

            // Write palette
            match section.get_palette() {
                Palette::Direct => {
                    // No palette data for Direct palette
                }
                Palette::Indirect {
                    bits_per_block: _,
                    palette,
                } => {
                    // Write palette length
                    temp_buffer.write_varint(palette.len() as i32);

                    // Write palette entries
                    for entry in palette {
                        temp_buffer.write_varint(*entry as i32);
                    }
                }
            }

            // Write data array length
            temp_buffer.write_varint(section.data_array.len() as i32);

            // Data array
            for value in &section.data_array {
                temp_buffer.write_i64(*value as i64);
            }

            // Light data
            for light in &section.block_light {
                temp_buffer.write_u8(*light);
            }
            if let Some(sky_light) = &section.sky_light {
                for light in sky_light {
                    temp_buffer.write_u8(*light);
                }
            }
        }

        // Write the size of the chunk data
        buffer.write_varint(temp_buffer.get_buffer().len() as i32);

        // Write the chunk data
        buffer.write_bytes_raw(temp_buffer.get_buffer());

        // Write block entities
        buffer.write_varint(self.block_entities.len() as i32);
        for entity in &self.block_entities {
            entity.write(buffer, "")?;
        }

        Ok(())
    }
}

impl ChunkSection {
    pub fn new() -> Self {
        ChunkSection {
            block_count: 0,
            bits_per_block: 4,
            data_array: vec![0; 256], // Initial size for 4 bits per block (4096 * 4 / 64)
            block_light: vec![0; 2048], // 16*16*16/2 (4 bits per block)
            sky_light: None,
            palette: Palette::new(4),
        }
    }

    /// Creates a new chunk section with the specified bits per block
    pub fn with_bits_per_block(bits: u8) -> Self {
        let data_array_size = (4096 * bits as usize + 63) / 64; // Round up to nearest 64 bits
        ChunkSection {
            block_count: 0,
            bits_per_block: bits,
            data_array: vec![0; data_array_size],
            block_light: vec![0; 2048],
            sky_light: None,
            palette: Palette::new(bits),
        }
    }

    /// Recalculates the non-air block count
    pub fn recalculate_block_count(&mut self) {
        let mut count = 0;
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let state = self.get_block_state(x, y, z);
                    if !state.is_air() {
                        count += 1;
                    }
                }
            }
        }
        self.block_count = count;
    }

    /// Sets whether sky light data is present
    pub fn set_sky_light(&mut self, has_sky_light: bool) {
        self.sky_light = if has_sky_light {
            Some(vec![0; 2048])
        } else {
            None
        };
    }

    /// Gets a reference to the current palette
    pub fn get_palette(&self) -> &Palette {
        &self.palette
    }

    /// Gets a mutable reference to the current palette
    pub fn get_palette_mut(&mut self) -> &mut Palette {
        &mut self.palette
    }

    /// Resizes the palette to use more bits per block if needed
    pub fn resize_palette_if_needed(&mut self) {
        if let Palette::Indirect {
            bits_per_block,
            ref palette,
        } = self.palette
        {
            let max_size = 1 << bits_per_block;
            if palette.len() >= max_size {
                // Need to resize
                let new_bits = if bits_per_block >= 8 {
                    // Switch to direct palette
                    14
                } else {
                    // Increase bits per block
                    bits_per_block + 1
                };

                // Create new section with increased bits
                let mut new_section = ChunkSection::with_bits_per_block(new_bits);

                // Copy all block states
                for y in 0..16 {
                    for z in 0..16 {
                        for x in 0..16 {
                            let state = self.get_block_state(x, y, z);
                            new_section.set_block_state(x, y, z, state);
                        }
                    }
                }

                // Update this section
                self.bits_per_block = new_bits;
                self.data_array = new_section.data_array;
                self.palette = new_section.palette;
            }
        }
    }

    pub fn set_block_state(&mut self, x: usize, y: usize, z: usize, state: BlockState) {
        let index = (y * 16 * 16) + (z * 16) + x;
        let bits = self.bits_per_block as usize;
        let long_index = (index * bits) / 64;
        let bit_offset = (index * bits) % 64;

        // Create mask for clearing old value
        let clear_mask = !((((1u64 << bits) - 1) as u64) << bit_offset);

        // Get the block state ID
        let state_id = match self.get_palette() {
            Palette::Direct => state.get_global_id() as u64,
            Palette::Indirect {
                bits_per_block: _,
                palette,
            } => {
                // Find or add to palette
                let palette_index = self.get_or_add_palette_entry(state);
                palette_index as u64
            }
        };

        // Write the new value
        self.data_array[long_index] &= clear_mask;
        self.data_array[long_index] |= (state_id << bit_offset) as u64;

        // Handle value spanning two longs
        if bit_offset + bits > 64 {
            let bits_in_next = bit_offset + bits - 64;
            let next_long_index = long_index + 1;
            if next_long_index < self.data_array.len() {
                let next_clear_mask = !((1u64 << bits_in_next) - 1);
                self.data_array[next_long_index] &= next_clear_mask;
                self.data_array[next_long_index] |= (state_id >> (bits - bits_in_next)) as u64;
            }
        }
    }

    pub fn get_block_state(&self, x: usize, y: usize, z: usize) -> BlockState {
        let index = (y * 16 * 16) + (z * 16) + x;
        let bits = self.bits_per_block as usize;
        let long_index = (index * bits) / 64;
        let bit_offset = (index * bits) % 64;

        // Read value potentially spanning two longs
        let mut value = (self.data_array[long_index] >> bit_offset) as u32;
        if bit_offset + bits > 64 {
            let bits_in_next = bit_offset + bits - 64;
            let next_long_index = long_index + 1;
            if next_long_index < self.data_array.len() {
                value |= ((self.data_array[next_long_index] & ((1u64 << bits_in_next) - 1))
                    << (bits - bits_in_next)) as u32;
            }
        }
        value &= (1u32 << bits) - 1;

        match self.get_palette() {
            Palette::Direct => BlockState::from_global_id(value),
            Palette::Indirect {
                bits_per_block: _,
                palette,
            } => {
                if value as usize >= palette.len() {
                    BlockState::default() // Invalid palette index
                } else {
                    BlockState::from_global_id(palette[value as usize])
                }
            }
        }
    }

    fn get_or_add_palette_entry(&mut self, state: BlockState) -> u32 {
        match &mut self.palette {
            Palette::Direct => state.get_global_id(),
            Palette::Indirect {
                bits_per_block: _,
                palette,
            } => {
                let global_id = state.get_global_id();
                match palette.iter().position(|&id| id == global_id) {
                    Some(index) => index as u32,
                    None => {
                        // Check if we need to resize before adding
                        self.resize_palette_if_needed();

                        // Try adding to palette again (it might be Direct now)
                        match &mut self.palette {
                            Palette::Direct => state.get_global_id(),
                            Palette::Indirect {
                                bits_per_block: _,
                                palette,
                            } => {
                                palette.push(global_id);
                                (palette.len() - 1) as u32
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Palette {
    pub fn new(bits_per_block: u8) -> Self {
        if bits_per_block >= 9 {
            Palette::Direct
        } else {
            let actual_bits = if bits_per_block <= 4 {
                4
            } else {
                bits_per_block
            };
            Palette::Indirect {
                bits_per_block: actual_bits,
                palette: Vec::new(),
            }
        }
    }

    pub fn get_bits_per_block(&self) -> u8 {
        match self {
            Palette::Direct => 14, // Current global palette size
            Palette::Indirect { bits_per_block, .. } => *bits_per_block,
        }
    }
}

impl ChunkDataPacket {
    /// Creates a new chunk data packet for a full chunk
    pub fn new_full_chunk(chunk: &ChunkColumn) -> Self {
        let mut primary_bit_mask = 0;
        let mut sections = Vec::new();

        // Convert chunk sections
        for y in 0..16 {
            if let Some(chunk_section) = chunk.get_section(y) {
                primary_bit_mask |= 1 << y;
                let mut section = ChunkSection::new();

                // Copy blocks
                for sy in 0..16 {
                    for sz in 0..16 {
                        for sx in 0..16 {
                            let state = chunk_section.get_block_state_at(sx, sy, sz);
                            section.set_block_state(sx, sy, sz, state);
                        }
                    }
                }

                section.recalculate_block_count();
                sections.push(section);
            }
        }

        // Create biomes array
        let mut biomes = Vec::with_capacity(1024);
        for y in 0..64 {
            for z in 0..4 {
                for x in 0..4 {
                    if let Some(biome) = chunk.get_biome(x * 4, y * 4, z * 4) {
                        biomes.push(biome);
                    } else {
                        biomes.push(127); // Void biome as fallback
                    }
                }
            }
        }

        ChunkDataPacket {
            chunk_x: chunk.get_x(),
            chunk_z: chunk.get_z(),
            full_chunk: true,
            primary_bit_mask,
            heightmaps: chunk.get_heightmaps().clone(),
            biomes: Some(biomes),
            sections,
            block_entities: chunk.get_block_entities().clone(),
        }
    }

    /// Creates a new chunk data packet for updating specific sections
    pub fn new_section_update(chunk: &ChunkColumn, sections_to_update: &[u8]) -> Self {
        let mut primary_bit_mask = 0;
        let mut sections = Vec::new();

        // Convert only the specified sections
        for &y in sections_to_update {
            if y < 16 && !chunk.is_section_empty(y as usize) {
                primary_bit_mask |= 1 << y;
                let mut section = ChunkSection::new();

                // Copy blocks
                for sy in 0..16 {
                    for sz in 0..16 {
                        for sx in 0..16 {
                            let state = chunk.get_block_state(sx, (y as usize * 16) + sy, sz);
                            section.set_block_state(sx, sy, sz, state);
                        }
                    }
                }

                section.recalculate_block_count();
                sections.push(section);
            }
        }

        ChunkDataPacket {
            chunk_x: chunk.get_x(),
            chunk_z: chunk.get_z(),
            full_chunk: false,
            primary_bit_mask,
            heightmaps: Vec::new(), // Empty for section updates
            biomes: None,           // No biomes for section updates
            sections,
            block_entities: Vec::new(), // No block entities for section updates
        }
    }

    /// Gets the chunk coordinates
    pub fn get_coordinates(&self) -> (i32, i32) {
        (self.chunk_x, self.chunk_z)
    }

    /// Checks if a section is present in the packet
    pub fn has_section(&self, y: u8) -> bool {
        if y >= 16 {
            return false;
        }
        (self.primary_bit_mask & (1 << y)) != 0
    }

    /// Gets a reference to a section if it exists
    pub fn get_section(&self, y: u8) -> Option<&ChunkSection> {
        if !self.has_section(y) {
            return None;
        }

        let mut section_index = 0;
        for i in 0..y {
            if self.has_section(i) {
                section_index += 1;
            }
        }

        self.sections.get(section_index as usize)
    }

    /// Gets a mutable reference to a section if it exists
    pub fn get_section_mut(&mut self, y: u8) -> Option<&mut ChunkSection> {
        if !self.has_section(y) {
            return None;
        }

        let mut section_index = 0;
        for i in 0..y {
            if self.has_section(i) {
                section_index += 1;
            }
        }

        self.sections.get_mut(section_index as usize)
    }
}

#[allow(dead_code)]
fn global_palette_id_for_state(state: BlockState) -> u32 {
    // TODO: Implement proper global palette lookup
    ((state.block_type as u32) << 16) | (state.properties as u32)
}

#[allow(dead_code)]
fn state_for_global_palette_id(id: u32) -> BlockState {
    // TODO: Implement proper global palette lookup
    BlockState {
        block_type: ((id >> 16) & 0xFFFF) as u16,
        properties: (id & 0xFFFF) as u16,
    }
}
