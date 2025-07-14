use crate::blend_file::BlendFile;
use crate::error::Result;
use std::path::Path;

pub fn debug_library_blocks<P: AsRef<Path>>(file_path: P) -> Result<()> {
    let blend_file = BlendFile::open(file_path.as_ref())?;

    println!("=== Debug Library Blocks ===");
    println!("File: {}", file_path.as_ref().display());
    println!(
        "Total blocks: {total_blocks}",
        total_blocks = blend_file.blocks.len()
    );

    for (i, block) in blend_file.blocks.iter().enumerate() {
        if &block.code[..2] == b"LI" {
            println!("\n--- LI Block #{block_num} ---", block_num = i + 1);
            println!(
                "Code: {code:?}",
                code = String::from_utf8_lossy(&block.code)
            );
            println!("Size: {size} bytes", size = block.data.len());
            println!("Data preview (first 256 bytes):");

            let preview_len = block.data.len().min(256);
            for (offset, chunk) in block.data[..preview_len].chunks(16).enumerate() {
                print!("{offset:04x}: ", offset = offset * 16);
                for byte in chunk {
                    print!("{byte:02x} ");
                }
                print!("  ");
                for &byte in chunk {
                    let c = if (32..=126).contains(&byte) {
                        byte as char
                    } else {
                        '.'
                    };
                    print!("{c}");
                }
                println!();
            }

            // Look for potential strings
            println!("\nPotential strings:");
            for offset in (0..block.data.len().saturating_sub(4)).step_by(4) {
                let slice = &block.data[offset..];
                if let Some(null_pos) = slice.iter().position(|&b| b == 0) {
                    if null_pos > 0 && null_pos < 256 {
                        let string_bytes = &slice[..null_pos];
                        if string_bytes.iter().all(|&b| (32..=126).contains(&b)) {
                            let s = String::from_utf8_lossy(string_bytes);
                            if !s.trim().is_empty() {
                                println!("  Offset {offset}: \"{s}\"");
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
