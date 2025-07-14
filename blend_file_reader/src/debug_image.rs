use crate::blend_file::BlendFile;
use crate::error::Result;
use std::path::Path;

pub fn debug_image_blocks<P: AsRef<Path>>(file_path: P) -> Result<()> {
    let blend_file = BlendFile::open(file_path.as_ref())?;

    println!("=== Debug Image Blocks ===");
    println!("File: {}", file_path.as_ref().display());

    for (i, block) in blend_file.blocks.iter().enumerate() {
        if &block.code[..2] == b"IM" {
            println!("\n--- IM Block #{block_num} ---", block_num = i + 1);
            println!(
                "Code: {code:?}",
                code = String::from_utf8_lossy(&block.code)
            );
            println!("Size: {size} bytes", size = block.data.len());

            // Look for filepath specifically
            if block.data.len() > 104 {
                let filepath_offset = 104;
                let start = filepath_offset;
                let end = block.data[start..]
                    .iter()
                    .position(|&b| b == 0)
                    .map(|pos| start + pos)
                    .unwrap_or(block.data.len().min(start + 256));

                if start < end {
                    let string_bytes = &block.data[start..end];
                    if let Ok(s) = String::from_utf8(string_bytes.to_vec()) {
                        println!("  filepath at offset {filepath_offset}: \"{s}\"");
                    }
                }
            }

            // Look for name
            if block.data.len() > 0 {
                let name_offset = 0;
                let start = name_offset;
                let end = block.data[start..]
                    .iter()
                    .position(|&b| b == 0)
                    .map(|pos| start + pos)
                    .unwrap_or(block.data.len().min(start + 64));

                if start < end {
                    let string_bytes = &block.data[start..end];
                    if let Ok(s) = String::from_utf8(string_bytes.to_vec()) {
                        println!("  name at offset {name_offset}: \"{s}\"");
                    }
                }
            }
        }
    }

    Ok(())
}
