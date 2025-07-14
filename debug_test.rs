use blend_file_reader::{BlendFile, Header};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let path = "tests/test_blend_files/linked_cube.blend";

    println!("Attempting to open: {}", path);

    match BlendFile::open(path) {
        Ok(blend) => {
            println!("Successfully opened blend file!");
            blend.print_summary();
        }
        Err(e) => {
            println!("Error: {}", e);

            // Try just reading the header
            use std::fs::File;
            use std::io::{BufReader, Seek};

            let file = File::open(path)?;
            let mut reader = BufReader::new(file);

            match Header::from_reader(&mut reader) {
                Ok(header) => {
                    println!("Header read successfully: {:?}", header);
                    let pos = reader.seek(std::io::SeekFrom::Current(0))?;
                    println!("Position after header: {}", pos);
                }
                Err(e) => {
                    println!("Header error: {}", e);
                }
            }
        }
    }

    Ok(())
}
