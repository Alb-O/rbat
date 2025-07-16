use blend_file_reader::BlendFile;

fn main() {
    let blend_path = "tests/test_blend_files/basic_file.blend";
    let blend_file = BlendFile::open(blend_path).expect("Failed to open blend file");

    println!("=== Blend File Summary ===");
    blend_file.print_summary();

    println!("\n=== OB Blocks Analysis ===");
    let ob_blocks = blend_file.get_blocks_by_type(b"OB");
    match ob_blocks {
        Ok(blocks) => {
            for (i, block) in blocks.iter().enumerate() {
                println!("OB Block {i}:");
                println!("  Size: {} bytes", block.size);
                println!("  Data offset: {}", block.data_offset);
                println!("  SDNA index: {}", block.sdna_index);
                println!("  Count: {}", block.count);

                // Print first 64 bytes of data
                let display_len = 64.min(block.data.len());
                println!(
                    "  First {} bytes: {:?}",
                    display_len,
                    &block.data[..display_len]
                );

                // Try to interpret as floats
                if block.data.len() >= 12 {
                    let mut floats = Vec::new();
                    for chunk in block.data.chunks_exact(4).take(3) {
                        let bytes = [chunk[0], chunk[1], chunk[2], chunk[3]];
                        floats.push(f32::from_le_bytes(bytes));
                    }
                    println!("  First 3 floats: {floats:?}");
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to get OB blocks: {e:?}");
        }
    }

    println!("\n=== Available DNA Structs ===");
    for name in blend_file.dna.structs.keys() {
        println!("  {name}");
    }
}
