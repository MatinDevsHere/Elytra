use elytra_nbt::Tag;

struct ChunkColumn {
    x: i32,
    z: i32,
    sections: [Option<ChunkSection>; 16],
    // TODO: For production, comment out if we always send void as the biome
    biomes: [i32; 1024],
    block_entities: Vec<Tag>, // Tag::Compound
    // TODO: Should be calculated on the fly
    heightmaps: Vec<Tag>, // Tag::Compound
}

struct ChunkSection {
    block_count: u16,
    palette: Palette,
    data: Vec<u64>, // Compacted data array (Vec of u64 longs)
}

enum Palette {
    Indirect {
        bits_per_block: u8,
        palette: Vec<u32>, // Global palette IDs (VarInts in the wire format)
    },
    Direct, // No internal data; uses global palette IDs directly
}
