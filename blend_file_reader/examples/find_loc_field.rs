use blend_file_reader::BlendFile;

fn main() {
    let blend_path = "tests/test_blend_files/basic_file.blend";
    let blend_file = BlendFile::open(blend_path).expect("Failed to open blend file");

    let ob_blocks = blend_file.get_blocks_by_type(b"OB");
    match ob_blocks {
        Ok(blocks) => {
            let ob_block = blocks.first().expect("No OB block found");

            println!("OB Block size: {} bytes", ob_block.size);

            // Search for the expected location values [2.0, 3.0, 5.0] in the data
            let target_values = [2.0f32, 3.0f32, 5.0f32];

            for i in 0..(ob_block.data.len() - 11) {
                let mut found = true;
                for j in 0..3 {
                    let start = i + j * 4;
                    let bytes = [
                        ob_block.data[start],
                        ob_block.data[start + 1],
                        ob_block.data[start + 2],
                        ob_block.data[start + 3],
                    ];
                    let value = f32::from_le_bytes(bytes);

                    if (value - target_values[j]).abs() > 0.001 {
                        found = false;
                        break;
                    }
                }

                if found {
                    println!("Found location [2.0, 3.0, 5.0] at offset {i}");

                    // Print some context around this location
                    let start = i.saturating_sub(16);
                    let end = (i + 28).min(ob_block.data.len());
                    println!(
                        "Context around offset {}: {:?}",
                        i,
                        &ob_block.data[start..end]
                    );

                    // Print as floats
                    print!("As floats: [");
                    for k in (start..end).step_by(4) {
                        if k + 4 <= ob_block.data.len() {
                            let bytes = [
                                ob_block.data[k],
                                ob_block.data[k + 1],
                                ob_block.data[k + 2],
                                ob_block.data[k + 3],
                            ];
                            let value = f32::from_le_bytes(bytes);
                            print!("{value:.2}, ");
                        }
                    }
                    println!("]");
                    break;
                }
            }

            // Also search for any non-zero float values
            println!("\nNon-zero float values in first 100 bytes:");
            for i in (0..100.min(ob_block.data.len())).step_by(4) {
                if i + 4 <= ob_block.data.len() {
                    let bytes = [
                        ob_block.data[i],
                        ob_block.data[i + 1],
                        ob_block.data[i + 2],
                        ob_block.data[i + 3],
                    ];
                    let value = f32::from_le_bytes(bytes);
                    if value.abs() > 0.001 {
                        println!("Offset {i}: {value:.3}");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to get OB blocks: {e:?}");
        }
    }
}
