use crate::block::{Block, BlockIterator};
use crate::dna::Dna;
use crate::error::Result;
use crate::header::Header;
use crate::library_link::{LibraryLink, LibraryLinkExtractor};
use memmap2::MmapOptions;
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct BlendFile {
    pub path: PathBuf,
    pub header: Header,
    pub dna: Dna,
    pub blocks: Vec<Block>,
}

impl BlendFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;

        // Memory map the file for efficient reading
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let mut reader = std::io::Cursor::new(&mmap);

        // Parse header
        let header = Header::from_reader(&mut reader)?;
        println!("DEBUG: Parsed header: {header:?}");

        // Parse all blocks
        let mut blocks = Vec::new();
        let block_iter = BlockIterator::new(&mut reader, &header);

        let mut block_count = 0;
        for block_result in block_iter {
            match block_result {
                Ok(block) => {
                    block_count += 1;
                    if block_count <= 5 {
                        println!(
                            "DEBUG: Block {block_count}: code={:?}, size={}",
                            String::from_utf8_lossy(&block.code),
                            block.size
                        );
                    }
                    blocks.push(block);
                }
                Err(e) => {
                    println!("DEBUG: Error reading block {block_count}: {e}");
                    return Err(e);
                }
            }
        }
        println!(
            "DEBUG: Total blocks parsed: {blocks_len}",
            blocks_len = blocks.len()
        );

        // Parse DNA
        let mut reader = std::io::Cursor::new(&mmap);
        let dna = Dna::from_reader(&mut reader, &header)?;
        println!("DEBUG: Parsed DNA");

        Ok(BlendFile {
            path,
            header,
            dna,
            blocks,
        })
    }

    pub fn get_library_links(&self) -> Result<Vec<LibraryLink>> {
        let extractor = LibraryLinkExtractor::new(&self.path);
        let mut links = extractor.extract_links(&self.blocks, &self.dna)?;
        extractor.resolve_relative_paths(&mut links)?;
        Ok(links)
    }

    pub fn get_blocks_by_type(&self, code: &[u8]) -> Vec<&Block> {
        self.blocks
            .iter()
            .filter(|b| &b.code[..code.len()] == code)
            .collect()
    }

    pub fn get_library_blocks(&self) -> Vec<&Block> {
        self.get_blocks_by_type(b"LI")
    }

    pub fn get_image_blocks(&self) -> Vec<&Block> {
        self.get_blocks_by_type(b"IM")
    }

    pub fn get_sound_blocks(&self) -> Vec<&Block> {
        self.get_blocks_by_type(b"SO")
    }

    pub fn get_movie_clip_blocks(&self) -> Vec<&Block> {
        self.get_blocks_by_type(b"MC")
    }

    pub fn get_mesh_blocks(&self) -> Vec<&Block> {
        self.get_blocks_by_type(b"ME")
    }

    pub fn get_material_blocks(&self) -> Vec<&Block> {
        self.get_blocks_by_type(b"MA")
    }

    pub fn get_texture_blocks(&self) -> Vec<&Block> {
        self.get_blocks_by_type(b"TE")
    }

    pub fn print_summary(&self) {
        println!("Blend File: {}", self.path.display());
        println!("Version: {version}", version = self.header.version);
        println!(
            "Pointer Size: {pointer_size:?}",
            pointer_size = self.header.pointer_size
        );
        println!(
            "Endianness: {endianness:?}",
            endianness = self.header.endianness
        );
        println!(
            "Total Blocks: {total_blocks}",
            total_blocks = self.blocks.len()
        );

        let block_counts: std::collections::HashMap<String, usize> = self
            .blocks
            .iter()
            .map(|b| (String::from_utf8_lossy(&b.code).into_owned(), 1))
            .fold(
                std::collections::HashMap::new(),
                |mut acc, (code, count)| {
                    *acc.entry(code).or_insert(0) += count;
                    acc
                },
            );

        println!("Block Types:");
        for (code, count) in block_counts {
            println!("  {code}: {count}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_blend_file_creation() {
        // Create a test blend file with valid header but incomplete data
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"BLENDER-v279RENDH").unwrap();

        // This should fail because the file is incomplete
        let result = BlendFile::open(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_block_filtering() {
        // This would need actual test blend files
        // For now, just test the structure
        let blend_file = BlendFile {
            path: PathBuf::from("test.blend"),
            header: crate::header::Header {
                magic: *b"BLENDER",
                pointer_size: crate::header::PointerSize::Bits64,
                endianness: crate::header::Endianness::Little,
                version: 279,
            },
            dna: crate::dna::Dna {
                structs: std::collections::HashMap::new(),
                type_sizes: std::collections::HashMap::new(),
            },
            blocks: vec![
                Block {
                    code: *b"LI\0\0",
                    size: 100,
                    old_memory_address: 0x1000,
                    sdna_index: 0,
                    count: 1,
                    data_offset: 100,
                    data: vec![0; 100],
                },
                Block {
                    code: *b"IM\0\0",
                    size: 200,
                    old_memory_address: 0x2000,
                    sdna_index: 1,
                    count: 1,
                    data_offset: 300,
                    data: vec![0; 200],
                },
            ],
        };

        let library_blocks = blend_file.get_library_blocks();
        assert_eq!(library_blocks.len(), 1);
        assert_eq!(&library_blocks[0].code[..2], b"LI");

        let image_blocks = blend_file.get_image_blocks();
        assert_eq!(image_blocks.len(), 1);
        assert_eq!(&image_blocks[0].code[..2], b"IM");
    }
}
