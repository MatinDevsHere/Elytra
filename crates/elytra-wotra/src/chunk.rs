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

    pub fn get_global_id(&self) -> u32 {
        global_palette_id_for_state(*self)
    }

    pub fn from_global_id(id: u32) -> Self {
        state_for_global_palette_id(id)
    }
}

impl Default for BlockState {
    fn default() -> Self {
        Self {
            block_type: 0,
            properties: 0,
        }
    }
}

pub struct ChunkColumn {
    x: i32,
    z: i32,
    sections: [Option<ChunkSection>; 16],
    biomes: [i32; 1024],      // 4x4x4 blocks for each section
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

    Tag::Compound(compound)
}

impl ChunkColumn {
    pub fn new(x: i32, z: i32) -> Self {
        ChunkColumn {
            x,
            z,
            sections: Default::default(),
            biomes: [127; 1024], // Initialize with 'Void' biome
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

    pub fn set_biome(&mut self, x: usize, y: usize, z: usize, biome_id: i32) {
        if x < 16 && y < 256 && z < 16 {
            let index = ((y >> 2) & 63) << 4 | ((z >> 2) & 3) << 2 | ((x >> 2) & 3);
            if index < 1024 {
                self.biomes[index] = biome_id;
            }
        }
    }

    pub fn get_biome(&self, x: usize, y: usize, z: usize) -> Option<i32> {
        if x < 16 && y < 256 && z < 16 {
            let index = ((y >> 2) & 63) << 4 | ((z >> 2) & 3) << 2 | ((x >> 2) & 3);
            if index < 1024 {
                Some(self.biomes[index])
            } else {
                None
            }
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

                // Ensure we don't access out of bounds
                if long_index >= 36 {
                    continue;
                }

                motion_blocking_data[long_index] |= (highest_y & 0x1FF) << bit_offset;
                // Only write to the next long if we're not at the last long and the value needs to span two longs
                if (index % 7) > 5 && long_index < 35 {
                    motion_blocking_data[long_index + 1] |=
                        (highest_y & 0x1FF) >> (63 - bit_offset);
                }
            }
        }

        // Clear existing heightmaps and add only the new one
        self.heightmaps.clear();
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
                // Find id tag
                let mut id = String::new();

                if let Some(Tag::String(id_val)) = compound.get("id") {
                    id = id_val.clone();
                }
                block_entity.write(&mut buffer, &id).unwrap();
            }
        }

        buffer
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }

    pub fn get_heightmaps(&self) -> &Vec<Tag> {
        &self.heightmaps
    }

    pub fn get_block_entities(&self) -> &Vec<Tag> {
        &self.block_entities
    }

    pub fn is_section_empty(&self, y: usize) -> bool {
        if y >= 16 {
            return true;
        }
        self.sections[y].is_none()
    }

    pub fn get_block_state(&self, x: usize, y: usize, z: usize) -> BlockState {
        let section_y = y >> 4;
        let section_local_y = y & 0xF;

        if let Some(section) = self.get_section(section_y) {
            section.get_block_state_at(x, section_local_y, z)
        } else {
            BlockState::default() // Air for empty sections
        }
    }

    pub fn set_block_state(&mut self, x: usize, y: usize, z: usize, state: BlockState) {
        let section_y = y >> 4;
        let section_local_y = y & 0xF;

        if let Some(section) = self.get_section_mut(section_y) {
            section.set_block_state(x, section_local_y, z, state);
        }
    }
}

impl ChunkSection {
    pub fn new() -> Self {
        ChunkSection {
            block_count: 0,
            palette: Palette::Indirect {
                bits_per_block: 4,
                palette: Vec::new(),
                state_to_index: HashMap::new(),
            },
            data: vec![0; 256], // Initial size for 4 bits per block
        }
    }

    pub fn get_block_state_at(&self, x: usize, y: usize, z: usize) -> BlockState {
        let index = (y << 8) | (z << 4) | x;
        match &self.palette {
            Palette::Direct => {
                let value = self.get_data_value(index);
                BlockState::from_global_id(value)
            }
            Palette::Indirect { palette, .. } => {
                let value = self.get_data_value(index) as usize;
                if value >= palette.len() {
                    BlockState::default() // Invalid palette index
                } else {
                    BlockState::from_global_id(palette[value])
                }
            }
        }
    }

    fn get_data_value(&self, index: usize) -> u32 {
        let bits_per_block = match &self.palette {
            Palette::Direct => 13,
            Palette::Indirect { bits_per_block, .. } => *bits_per_block,
        };

        let bits_per_value = bits_per_block as usize;
        let values_per_long = 64 / bits_per_value;
        let long_index = index / values_per_long;
        let bit_offset = (index % values_per_long) * bits_per_value;

        if long_index >= self.data.len() {
            return 0;
        }

        let value = self.data[long_index];
        let mask = (1u64 << bits_per_value) - 1;
        ((value >> bit_offset) & mask) as u32
    }

    fn set_data_value(&mut self, index: usize, value: u32) {
        let bits_per_block = match &self.palette {
            Palette::Direct => 13,
            Palette::Indirect { bits_per_block, .. } => *bits_per_block,
        };

        let bits_per_value = bits_per_block as usize;
        let values_per_long = 64 / bits_per_value;
        let long_index = index / values_per_long;
        let bit_offset = (index % values_per_long) * bits_per_value;

        if long_index >= self.data.len() {
            self.data.resize(long_index + 1, 0);
        }

        let mask = (1u64 << bits_per_value) - 1;
        let value = (value as u64) & mask;
        self.data[long_index] &= !(mask << bit_offset);
        self.data[long_index] |= value << bit_offset;
    }

    pub fn set_block_state(&mut self, x: usize, y: usize, z: usize, state: BlockState) {
        let index = (y << 8) | (z << 4) | x;
        match &mut self.palette {
            Palette::Direct => {
                self.set_data_value(index, state.get_global_id());
            }
            Palette::Indirect {
                bits_per_block,
                palette,
                state_to_index,
            } => {
                let global_id = state.get_global_id();
                let palette_index = if let Some(&index) = state_to_index.get(&state) {
                    index
                } else {
                    let index = palette.len() as u32;
                    palette.push(global_id);
                    state_to_index.insert(state, index);
                    index
                };
                self.set_data_value(index, palette_index);
            }
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
                bits_per_block: _,
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
    pub fn new(bits_per_block: u8) -> Self {
        if bits_per_block >= 9 {
            Palette::Direct
        } else {
            Palette::Indirect {
                bits_per_block,
                palette: Vec::new(),
                state_to_index: HashMap::new(),
            }
        }
    }

    pub fn bits_per_block(&self) -> u8 {
        match self {
            Palette::Direct => 13,
            Palette::Indirect { bits_per_block, .. } => *bits_per_block,
        }
    }

    pub fn state_for_id(&self, id: u32) -> BlockState {
        match self {
            Palette::Direct => BlockState::from_global_id(id),
            Palette::Indirect { palette, .. } => {
                if id as usize >= palette.len() {
                    BlockState::default()
                } else {
                    BlockState::from_global_id(palette[id as usize])
                }
            }
        }
    }

    pub fn id_for_state(&mut self, state: BlockState) -> u32 {
        match self {
            Palette::Direct => state.get_global_id(),
            Palette::Indirect {
                palette,
                state_to_index,
                ..
            } => {
                if let Some(&index) = state_to_index.get(&state) {
                    index
                } else {
                    let index = palette.len() as u32;
                    palette.push(state.get_global_id());
                    state_to_index.insert(state, index);
                    index
                }
            }
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
