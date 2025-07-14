use blend_file_reader::BlendFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Blend File Reader Demo ===");
    println!("This demonstrates how to read Blender .blend files and extract library links.");
    println!();

    // Create a simple test file with valid header
    let test_file = "test_demo.blend";
    let header = b"BLENDER-v279RENDH";

    // Create minimal file structure
    let mut content = Vec::new();
    content.extend_from_slice(header);

    // Add some padding to make it a valid file size
    content.resize(1024, 0);

    std::fs::write(test_file, &content)?;

    println!("Created test file: {test_file}");

    // Try to open the file
    match BlendFile::open(test_file) {
        Ok(blend_file) => {
            println!("Successfully opened blend file!");
            println!("File: {}", blend_file.path.display());
            println!("Version: {}", blend_file.header.version);
            println!("Pointer Size: {:?}", blend_file.header.pointer_size);
            println!("Endianness: {:?}", blend_file.header.endianness);
            println!("Total Blocks: {}", blend_file.blocks.len());

            // Get library links
            match blend_file.get_library_links() {
                Ok(links) => {
                    println!("Library Links Found: {}", links.len());
                    for link in links {
                        println!(
                            "  - {}: {} (relative: {})",
                            link.block_type, link.path, link.is_relative
                        );
                    }
                }
                Err(e) => println!("Error getting library links: {e}"),
            }
        }
        Err(e) => {
            println!("Error opening blend file: {e}");
            println!("This is expected for incomplete test files.");
        }
    }

    // Clean up
    let _ = std::fs::remove_file(test_file);

    Ok(())
}
