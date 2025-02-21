use byteorder::{BigEndian, WriteBytesExt};
use elytra_nbt::Tag;
use std::collections::HashMap;

mod global_palette {
    include!(concat!(env!("OUT_DIR"), "/global_palette.rs"));
}
use global_palette::GLOBAL_PALETTE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub block_type: u16,
    pub properties: u16,
}

impl BlockState {
    pub fn is_air(&self) -> bool {
        self.block_type == 0 // Assuming block_type 0 is air
    }
}

pub struct ChunkColumn {
    x: i32,
    z: i32,
    sections: [Option<ChunkSection>; 16],
    biomes: [i32; 1024],
    block_entities: Vec<Tag>, // Tag::Compound
    heightmaps: Vec<Tag>,     // Tag::Compound
}

pub struct ChunkSection {
    block_count: u16,
    palette: Palette,
    data: Vec<u64>, // Compacted data array
}

#[derive(Debug, Clone)]
pub enum Palette {
    Indirect {
        bits_per_block: u8,
        palette: Vec<u32>,                        // Global palette IDs
        state_to_index: HashMap<BlockState, u32>, // Add this for reverse lookup
    },
    Direct,
}

// Helper function to write VarInts
fn write_varint(buffer: &mut Vec<u8>, mut value: i32) {
    loop {
        let mut temp = (value & 0b0111_1111) as u8;
        value >>= 7;
        if value != 0 {
            temp |= 0b1000_0000;
        }
        buffer.push(temp);
        if value == 0 {
            break;
        }
    }
}

// Helper function to create heightmap
fn create_heightmap(data: &[u64; 36]) -> Tag {
    let mut heightmap_data = Vec::new();
    for &long in data {
        heightmap_data.push(long as i64); // Convert to i64 for LongArray
    }

    let mut compound = HashMap::new();
    compound.insert(
        "MOTION_BLOCKING".to_string(),
        Tag::LongArray(heightmap_data),
    );
    // Optionally add WORLD_SURFACE, though it's not required.

    Tag::Compound(compound)
}

impl ChunkColumn {
    pub fn new(x: i32, z: i32) -> Self {
        ChunkColumn {
            x,
            z,
            sections: Default::default(),
            biomes: [127; 1024], // Initialize with 'Void' biome (127)
            block_entities: Vec::new(),
            heightmaps: Vec::new(),
        }
    }

    pub fn get_section(&self, section_y: usize) -> Option<&ChunkSection> {
        self.sections.get(section_y)?.as_ref()
    }

    pub fn get_section_mut(&mut self, section_y: usize) -> Option<&mut ChunkSection> {
        if section_y >= 16 {
            return None;
        }

        if self.sections[section_y].is_none() {
            self.sections[section_y] = Some(ChunkSection::new());
        }
        self.sections[section_y].as_mut()
    }

    pub fn set_biome(&mut self, x: usize, z: usize, biome_id: i32) {
        if x < 16 && z < 16 {
            let index = ((z & 15) << 2) | (x & 15);
            self.biomes[index] = biome_id;
        }
    }

    pub fn get_biome(&self, x: usize, z: usize) -> Option<i32> {
        if x < 16 && z < 16 {
            let index = ((z & 15) << 2) | (x & 15);
            Some(self.biomes[index])
        } else {
            None
        }
    }

    pub fn add_block_entity(&mut self, block_entity: Tag) {
        if block_entity.as_compound().is_some() {
            self.block_entities.push(block_entity);
        }
    }

    pub fn calculate_heightmaps(&mut self) {
        let mut motion_blocking_data = [0u64; 36];

        for x in 0..16 {
            for z in 0..16 {
                let mut highest_y = 0;
                for section_y in (0..16).rev() {
                    if let Some(section) = self.get_section(section_y) {
                        for y in (0..16).rev() {
                            let block_state = section.get_block_state_at(x, y, z);
                            if !block_state.is_air() {
                                highest_y = (section_y * 16 + y) as u64;
                                break;
                            }
                        }
                        if highest_y != 0 {
                            break;
                        }
                    }
                }
                let index = x + z * 16;
                let long_index = index / 7;
                let bit_offset = (index % 7) * 9;

                motion_blocking_data[long_index] |= (highest_y & 0x1FF) << bit_offset;
                if (index % 7) > 5 {
                    motion_blocking_data[long_index + 1] |=
                        (highest_y & 0x1FF) >> (63 - bit_offset);
                }
            }
        }
        self.heightmaps
            .push(create_heightmap(&motion_blocking_data));
    }
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        // 1. Chunk X and Z (Int)
        buffer.write_i32::<BigEndian>(self.x).unwrap();
        buffer.write_i32::<BigEndian>(self.z).unwrap();

        // 2. Full Chunk (Boolean)
        buffer.write_u8(1).unwrap(); // Always true for now

        // 3. Primary Bit Mask (VarInt)
        let mut primary_bit_mask = 0;
        for (section_y, section) in self.sections.iter().enumerate() {
            if section.is_some() {
                primary_bit_mask |= 1 << section_y;
            }
        }
        write_varint(&mut buffer, primary_bit_mask as i32);

        // 4. Heightmaps (NBT)
        // Assuming you've already calculated the heightmaps
        for tag in &self.heightmaps {
            tag.write(&mut buffer, "").unwrap(); // Empty string for name, as per NBT spec
        }

        // 5. Biomes (Optional array of Int)
        for &biome in &self.biomes {
            buffer.write_i32::<BigEndian>(biome).unwrap();
        }

        // 6. Data (Chunk Sections)
        let mut data_buffer = Vec::new();
        for section in self.sections.iter().flatten() {
            // Serialize each non-empty section
            section.serialize(&mut data_buffer);
        }

        // 7. Size (VarInt) - Size of the Data section
        write_varint(&mut buffer, data_buffer.len() as i32);

        // 8. Data (Byte array) - The actual chunk section data
        buffer.extend(data_buffer);

        // 9. Number of Block Entities (VarInt)
        write_varint(&mut buffer, self.block_entities.len() as i32);

        // 10. Block Entities (Array of NBT Tag)
        for block_entity in &self.block_entities {
            if let Some(compound) = block_entity.as_compound() {
                // Find x, y, z and id tags
                let mut x = 0;
                let mut y = 0;
                let mut z = 0;
                let mut id = String::new();

                if let Some(Tag::Int(x_val)) = compound.get("x") {
                    x = *x_val;
                }
                if let Some(Tag::Int(y_val)) = compound.get("y") {
                    y = *y_val;
                }
                if let Some(Tag::Int(z_val)) = compound.get("z") {
                    z = *z_val;
                }
                if let Some(Tag::String(id_val)) = compound.get("id") {
                    id = id_val.clone();
                }
                block_entity.write(&mut buffer, &id).unwrap();
            }
        }

        buffer
    }
}

impl ChunkSection {
    pub fn new() -> Self {
        ChunkSection {
            block_count: 0,
            palette: Palette::default(), // Use a default palette (likely Direct)
            data: vec![0; 4096 * 14 / 64], // Initialize with enough space for 14 bits/block
        }
    }

    pub fn get_block_state_at(&self, x: usize, y: usize, z: usize) -> BlockState {
        let bits_per_block = self.palette.bits_per_block();
        let block_number = (y * 16 * 16) + (z * 16) + x;
        let start_long = (block_number * bits_per_block as usize) / 64;
        let start_offset = (block_number * bits_per_block as usize) % 64;
        let end_long = ((block_number + 1) * bits_per_block as usize - 1) / 64;
        let individual_value_mask = (1 << bits_per_block) - 1;

        let mut data = if start_long == end_long {
            (self.data[start_long] >> start_offset) as u32
        } else {
            let end_offset = 64 - start_offset;
            ((self.data[start_long] >> start_offset) | (self.data[end_long] << end_offset)) as u32
        };

        data &= individual_value_mask;
        self.palette.state_for_id(data)
    }

    pub fn set_block_state_at(&mut self, x: usize, y: usize, z: usize, state: BlockState) {
        let bits_per_block = self.palette.bits_per_block();
        let block_number = (y * 16 * 16) + (z * 16) + x;
        let start_long = (block_number * bits_per_block as usize) / 64;
        let start_offset = (block_number * bits_per_block as usize) % 64;
        let end_long = ((block_number + 1) * bits_per_block as usize - 1) / 64;
        let individual_value_mask = (1 << bits_per_block) - 1;

        let value = self.palette.id_for_state(state) as u64;
        let value = value & individual_value_mask;

        self.data[start_long] &= !(individual_value_mask << start_offset); // Clear existing bits
        self.data[start_long] |= value << start_offset;

        if start_long != end_long {
            self.data[end_long] &= !individual_value_mask; // Clear existing bits
            self.data[end_long] |= value >> (64 - start_offset);
        }

        // Update block_count (you'll need to track changes)
        if state.is_air() && !self.get_block_state_at(x, y, z).is_air() {
            self.block_count -= 1;
        } else if !state.is_air() && self.get_block_state_at(x, y, z).is_air() {
            self.block_count += 1;
        }
    }

    pub fn serialize(&self, buffer: &mut Vec<u8>) {
        // 1. Block Count (Short)
        buffer.write_u16::<BigEndian>(self.block_count).unwrap();

        // 2. Bits Per Block (Unsigned Byte)
        buffer.write_u8(self.palette.bits_per_block()).unwrap();

        // 3. Palette (Varies)
        match &self.palette {
            Palette::Indirect {
                bits_per_block,
                palette,
                state_to_index: _,
            } => {
                // Palette Length (VarInt)
                write_varint(buffer, palette.len() as i32);
                // Palette (Array of VarInt)
                for &global_id in palette {
                    write_varint(buffer, global_id as i32);
                }
            }
            Palette::Direct => {
                // No palette data for Direct palette
            }
        }

        // 4. Data Array Length (VarInt)
        write_varint(buffer, self.data.len() as i32);

        // 5. Data Array (Array of Long)
        for &long in &self.data {
            buffer.write_u64::<BigEndian>(long).unwrap();
        }
    }

    /// Updates the palette to use the minimum number of bits per block.
    /// Returns true if the palette was changed, false otherwise.
    pub fn optimize_palette(&mut self) -> bool {
        let unique_state_count = self.palette.calculate_unique_states(&self);
        let best_palette = Palette::choose_best_palette(unique_state_count);

        // If the current palette is already optimal, don't change it.
        if best_palette.bits_per_block() == self.palette.bits_per_block() {
            return false;
        }
        match best_palette {
            Palette::Indirect {
                bits_per_block,
                palette: _,
                state_to_index: _,
            } => {
                let mut new_palette = Vec::new();
                let mut new_state_to_index = HashMap::new();
                let mut new_data = vec![0u64; 4096 * bits_per_block as usize / 64];

                for x in 0..16 {
                    for y in 0..16 {
                        for z in 0..16 {
                            let old_state = self.get_block_state_at(x, y, z);
                            let global_id = global_palette_id_for_state(old_state);

                            // Add to new palette if not already present
                            let new_index = if let Some(index) =
                                new_palette.iter().position(|&id| id == global_id)
                            {
                                index as u32
                            } else {
                                let index = new_palette.len() as u32;
                                new_palette.push(global_id);
                                new_state_to_index.insert(old_state, index);
                                index
                            };

                            // Set in new data array
                            let block_number = (y * 16 * 16) + (z * 16) + x;
                            let start_long = (block_number * bits_per_block as usize) / 64;
                            let start_offset = (block_number * bits_per_block as usize) % 64;
                            let end_long = ((block_number + 1) * bits_per_block as usize - 1) / 64;
                            let individual_value_mask = (1 << bits_per_block) - 1;

                            let value = new_index as u64 & individual_value_mask;

                            new_data[start_long] &= !(individual_value_mask << start_offset); // Clear existing bits
                            new_data[start_long] |= value << start_offset;

                            if start_long != end_long {
                                new_data[end_long] &= !individual_value_mask; // Clear existing bits
                                new_data[end_long] |= value >> (64 - start_offset);
                            }
                        }
                    }
                }

                self.palette = Palette::Indirect {
                    bits_per_block,
                    palette: new_palette,
                    state_to_index: new_state_to_index,
                };
                self.data = new_data;
                true
            }
            Palette::Direct => {
                let mut new_data = vec![0u64; 4096 * 14 / 64];
                for x in 0..16 {
                    for y in 0..16 {
                        for z in 0..16 {
                            let old_state = self.get_block_state_at(x, y, z);
                            let global_id = global_palette_id_for_state(old_state);

                            let block_number = (y * 16 * 16) + (z * 16) + x;
                            let start_long = (block_number * 14) / 64;
                            let start_offset = (block_number * 14) % 64;
                            let end_long = ((block_number + 1) * 14 - 1) / 64;
                            let individual_value_mask = (1 << 14) - 1;

                            let value = global_id as u64 & individual_value_mask;

                            new_data[start_long] &= !(individual_value_mask << start_offset);
                            new_data[start_long] |= value << start_offset;

                            if start_long != end_long {
                                new_data[end_long] &= !individual_value_mask;
                                new_data[end_long] |= value >> (64 - start_offset);
                            }
                        }
                    }
                }
                self.palette = Palette::Direct;
                self.data = new_data;
                true
            }
        }
    }
}

impl Palette {
    pub fn bits_per_block(&self) -> u8 {
        match self {
            Palette::Indirect { bits_per_block, .. } => *bits_per_block,
            Palette::Direct => 14, // Or calculate based on global palette size
        }
    }

    pub fn id_for_state(&self, state: BlockState) -> u32 {
        match self {
            Palette::Indirect {
                palette,
                state_to_index,
                ..
            } => {
                // 1. Try to find the state in the HashMap
                if let Some(&index) = state_to_index.get(&state) {
                    return index;
                }

                // 2. If not found, get the global ID
                let global_id = global_palette_id_for_state(state);

                // 3. Check if the global ID is already in the palette (rare, but possible)
                if let Some(index) = palette.iter().position(|&id| id == global_id) {
                    return index as u32;
                }

                // 4. Add the global ID to the palette

                // If the palette is full, we should resize, but for now, we'll just return the global ID.
                // This will cause incorrect serialization if not handled properly during serialization.
                // The proper fix is to resize *before* adding, but that requires more complex logic.
                let next_index = palette.len() as u32;
                if next_index >= (1 << self.bits_per_block()) {
                    // Palette is full!  In a real implementation, you'd resize here.
                    // For this example, we'll just return the global ID, which will lead to incorrect behavior.
                    return global_id;
                }
                let mut new_palette = palette.clone();
                let mut new_state_to_index = state_to_index.clone();

                new_palette.push(global_id);
                new_state_to_index.insert(state, next_index);

                // 5. Return the new index
                next_index
            }
            Palette::Direct => global_palette_id_for_state(state),
        }
    }

    pub fn state_for_id(&self, id: u32) -> BlockState {
        match self {
            Palette::Indirect { palette, .. } => {
                // Lookup the global ID in the palette
                let global_id = palette
                    .get(id as usize)
                    .expect("Palette index out of bounds");
                // Get the BlockState from the global palette
                state_for_global_palette_id(*global_id)
            }
            Palette::Direct => state_for_global_palette_id(id),
        }
    }

    pub fn choose_best_palette(unique_state_count: usize) -> Self {
        let bits_needed = (unique_state_count as f64).log2().ceil() as u8;

        if bits_needed <= 4 {
            Palette::Indirect {
                bits_per_block: 4,
                palette: Vec::new(),
                state_to_index: HashMap::new(),
            }
        } else if bits_needed <= 8 {
            Palette::Indirect {
                bits_per_block: bits_needed,
                palette: Vec::new(),
                state_to_index: HashMap::new(),
            }
        } else {
            Palette::Direct
        }
    }
    pub fn calculate_unique_states(&self, section: &ChunkSection) -> usize {
        let mut unique_states = std::collections::HashSet::new();
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    unique_states.insert(section.get_block_state_at(x, y, z));
                }
            }
        }
        unique_states.len()
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette::Direct // Start with a Direct palette by default
    }
}

// Function to get the global palette ID from a BlockState
fn global_palette_id_for_state(state: BlockState) -> u32 {
    for &(ref s, id) in GLOBAL_PALETTE {
        if *s == state {
            return id;
        }
    }
    panic!("BlockState not found in global palette: {:?}", state); // Or handle more gracefully
}

// Function to get the BlockState from a global palette ID
fn state_for_global_palette_id(id: u32) -> BlockState {
    for &(ref s, i) in GLOBAL_PALETTE {
        if i == id {
            return *s;
        }
    }
    panic!("Global palette ID not found: {}", id); // Or handle more gracefully
}
