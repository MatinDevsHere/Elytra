use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct BlockState {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    #[serde(rename = "type")]
    state_type: String,
    num_values: usize,
    #[allow(dead_code)]
    #[serde(default)] // Handle missing values
    values: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Block {
    #[allow(dead_code)]
    id: u32,
    name: String,
    #[serde(rename = "minStateId")]
    min_state_id: u32,
    #[serde(rename = "maxStateId")]
    max_state_id: u32,
    states: Vec<BlockState>,
    #[allow(dead_code)]
    #[serde(rename = "defaultState")]
    default_state: u32,
}

fn main() {
    let blocks_json_path = "blocks.json"; // Use the correct relative path
    let blocks_json = fs::read_to_string(blocks_json_path).expect("Failed to read blocks.json");

    let blocks: Vec<Block> =
        serde_json::from_str(&blocks_json).expect("Failed to parse blocks.json");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("global_palette.rs");
    let mut out_file = File::create(&dest_path).expect("Failed to create global_palette.rs");

    writeln!(&mut out_file, "use crate::chunk::{{BlockState}};").unwrap();

    writeln!(
        &mut out_file,
        "pub static GLOBAL_PALETTE: &[(BlockState, u32)] = &["
    )
    .unwrap();

    let mut next_block_type_id = 0;
    let mut block_name_to_id: HashMap<String, u16> = HashMap::new();

    for block in blocks {
        // Assign a unique block_type ID
        let block_type_id = *block_name_to_id
            .entry(block.name.clone())
            .or_insert_with(|| {
                let id = next_block_type_id;
                next_block_type_id += 1;
                id
            });

        for state_id in block.min_state_id..=block.max_state_id {
            // Calculate properties
            let mut properties: u16 = 0;
            let mut state_offset = state_id - block.min_state_id;

            for block_state in block.states.iter().rev() {
                let num_values = block_state.num_values as u32; // Cast to u32
                let property_value = state_offset % num_values;
                state_offset /= num_values;

                properties = (properties << ((num_values as f64).log2().ceil() as u16))
                    | (property_value as u16);
            }

            let block_state_str = format!(
                "BlockState {{ block_type: {}, properties: {} }}",
                block_type_id, properties
            );

            writeln!(&mut out_file, "    ({}, {}),", block_state_str, state_id).unwrap();
        }
    }

    writeln!(&mut out_file, "];").unwrap();
    println!("cargo:rerun-if-changed={}", blocks_json_path);
}
