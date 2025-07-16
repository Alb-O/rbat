use crate::block::{Block, BlockIterator};
use crate::dna::Dna;
use crate::error::Result;
use crate::header::Header;
use crate::library_link::{LibraryLink, LibraryLinkExtractor};
use flate2::read::{GzDecoder, ZlibDecoder};
use memmap2::Mmap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use zstd::stream::read::Decoder as ZstdDecoder;

#[derive(Debug)]
pub struct BlendFile {
    pub path: PathBuf,
    pub header: Header,
    pub dna: Dna,
    pub blocks: Vec<Block>,
    pub mmap: Option<Mmap>,
    pub file: Option<File>,
}

impl BlendFile {
    /// Detects compression type and returns decompressed data if needed
    fn decompress_if_needed<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
        let mut file = File::open(&path)?;
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        file.seek(SeekFrom::Start(0))?;
        // Zlib: 0x78 0x9C or 0x78 0x01 or 0x78 0xDA
        // Zstd: 0x28 0xB5 0x2F 0xFD
        // Gzip: 0x1f 0x8b 0x08 0x00
        if magic[..2] == [0x78, 0x9C] || magic[..2] == [0x78, 0x01] || magic[..2] == [0x78, 0xDA] {
            let mut decoder = ZlibDecoder::new(file);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            Ok(decompressed)
        } else if magic == [0x28, 0xB5, 0x2F, 0xFD] {
            let mut decoder = ZstdDecoder::new(file)?;
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            Ok(decompressed)
        } else if magic == [0x1f, 0x8b, 0x08, 0x00] {
            let mut decoder = GzDecoder::new(file);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            Ok(decompressed)
        } else {
            // Not compressed
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            Ok(data)
        }
    }

    /// Open a blend file in read-only mode
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let data = Self::decompress_if_needed(&path)?;
        let mut reader = std::io::Cursor::new(&data);

        // Parse header
        let header = Header::from_reader(&mut reader)?;

        // Parse all blocks
        let mut blocks = Vec::new();
        let block_iter = BlockIterator::new(&mut reader, &header);

        for block_result in block_iter {
            match block_result {
                Ok(block) => blocks.push(block),
                Err(e) => return Err(e),
            }
        }

        // Parse DNA
        let mut reader = std::io::Cursor::new(&data);
        let dna = Dna::from_reader(&mut reader, &header)?;

        Ok(BlendFile {
            path,
            header,
            dna,
            blocks,
            mmap: None,
            file: None,
        })
    }

    /// Open a blend file in read+write mode for modification
    pub fn open_read_write<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let data = Self::decompress_if_needed(&path)?;
        let file = OpenOptions::new().read(true).write(true).open(&path)?;
        let mut reader = std::io::Cursor::new(&data);

        // Parse header
        let header = Header::from_reader(&mut reader)?;

        // Parse all blocks
        let mut blocks = Vec::new();
        let block_iter = BlockIterator::new(&mut reader, &header);

        for block_result in block_iter {
            match block_result {
                Ok(block) => blocks.push(block),
                Err(e) => return Err(e),
            }
        }

        // Parse DNA
        let mut reader = std::io::Cursor::new(&data);
        let dna = Dna::from_reader(&mut reader, &header)?;

        Ok(BlendFile {
            path,
            header,
            dna,
            blocks,
            mmap: None,
            file: Some(file),
        })
    }

    /// Get library links from the blend file
    pub fn get_library_links(&self) -> Result<Vec<LibraryLink>> {
        let extractor = LibraryLinkExtractor::new(&self.path);
        let mut links = extractor.extract_links(&self.blocks, &self.dna)?;
        extractor.resolve_relative_paths(&mut links)?;
        Ok(links)
    }

    /// Get blocks by type code
    pub fn get_blocks_by_type(&self, code: &[u8]) -> Result<Vec<&Block>> {
        Ok(self
            .blocks
            .iter()
            .filter(|b| &b.code[..code.len()] == code)
            .collect())
    }

    /// Get mutable blocks by type code
    pub fn get_blocks_by_type_mut(&mut self, code: &[u8]) -> Result<Vec<&mut Block>> {
        Ok(self
            .blocks
            .iter_mut()
            .filter(|b| &b.code[..code.len()] == code)
            .collect())
    }

    /// Get library blocks
    pub fn get_library_blocks(&self) -> Result<Vec<&Block>> {
        self.get_blocks_by_type(b"LI")
    }

    /// Get mutable library blocks
    pub fn get_library_blocks_mut(&mut self) -> Result<Vec<&mut Block>> {
        self.get_blocks_by_type_mut(b"LI")
    }

    /// Get image blocks
    pub fn get_image_blocks(&self) -> Result<Vec<&Block>> {
        self.get_blocks_by_type(b"IM")
    }

    /// Get mutable image blocks
    pub fn get_image_blocks_mut(&mut self) -> Result<Vec<&mut Block>> {
        self.get_blocks_by_type_mut(b"IM")
    }

    /// Get sound blocks
    pub fn get_sound_blocks(&self) -> Result<Vec<&Block>> {
        self.get_blocks_by_type(b"SO")
    }

    /// Get mutable sound blocks
    pub fn get_sound_blocks_mut(&mut self) -> Result<Vec<&mut Block>> {
        self.get_blocks_by_type_mut(b"SO")
    }

    /// Get movie clip blocks
    pub fn get_movie_clip_blocks(&self) -> Result<Vec<&Block>> {
        self.get_blocks_by_type(b"MC")
    }

    /// Get mutable movie clip blocks
    pub fn get_movie_clip_blocks_mut(&mut self) -> Result<Vec<&mut Block>> {
        self.get_blocks_by_type_mut(b"MC")
    }

    /// Get mesh blocks
    pub fn get_mesh_blocks(&self) -> Result<Vec<&Block>> {
        self.get_blocks_by_type(b"ME")
    }

    /// Get mutable mesh blocks
    pub fn get_mesh_blocks_mut(&mut self) -> Result<Vec<&mut Block>> {
        self.get_blocks_by_type_mut(b"ME")
    }

    /// Get material blocks
    pub fn get_material_blocks(&self) -> Result<Vec<&Block>> {
        self.get_blocks_by_type(b"MA")
    }

    /// Get mutable material blocks
    pub fn get_material_blocks_mut(&mut self) -> Result<Vec<&mut Block>> {
        self.get_blocks_by_type_mut(b"MA")
    }

    /// Get texture blocks
    pub fn get_texture_blocks(&self) -> Result<Vec<&Block>> {
        self.get_blocks_by_type(b"TE")
    }

    /// Get mutable texture blocks
    pub fn get_texture_blocks_mut(&mut self) -> Result<Vec<&mut Block>> {
        self.get_blocks_by_type_mut(b"TE")
    }

    /// Write changes back to the file
    pub fn save(&mut self) -> Result<()> {
        if let Some(ref mut file) = self.file {
            // We need to reconstruct the file with our modified blocks
            // For now, this is a simplified implementation that rewrites the entire file
            let mut writer = std::io::Cursor::new(Vec::new());

            // Write header
            self.header.write_to_writer(&mut writer)?;

            // Write all blocks
            for block in &self.blocks {
                block.write_to_writer(&mut writer, &self.header)?;
            }

            // Write DNA
            self.dna.write_to_writer(&mut writer)?;

            // Write the data back to the file
            file.set_len(0)?;
            file.seek(SeekFrom::Start(0))?;
            file.write_all(&writer.into_inner())?;

            Ok(())
        } else {
            Err(
                std::io::Error::other("File not opened in write mode")
                    .into(),
            )
        }
    }

    /// Close the file and release resources
    pub fn close(&mut self) {
        self.mmap = None;
        self.file = None;
    }

    /// Check if the file is compressed (placeholder for now)
    pub fn is_compressed(&self) -> bool {
        // TODO: Implement compression detection
        false
    }

    /// Print a summary of the blend file
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
                    data_offset: 200,
                    data: vec![0; 200],
                },
            ],
            mmap: None,
            file: None,
        };

        let library_blocks = blend_file.get_library_blocks();
        match library_blocks {
            Ok(blocks) => {
                assert_eq!(blocks.len(), 1);
                assert_eq!(&blocks[0].code[..2], b"LI");
            }
            Err(e) => panic!("Failed to get library blocks: {e:?}"),
        }

        let image_blocks = blend_file.get_image_blocks();
        match image_blocks {
            Ok(blocks) => {
                assert_eq!(blocks.len(), 1);
                assert_eq!(&blocks[0].code[..2], b"IM");
            }
            Err(e) => panic!("Failed to get image blocks: {e:?}"),
        }
    }
}
