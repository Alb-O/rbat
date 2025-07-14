use crate::block::Block;
use crate::dna::Dna;
use crate::error::{BlendFileError, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LibraryLink {
    pub path: String,
    pub absolute_path: Option<String>,
    pub block_type: String,
    pub block_name: Option<String>,
    pub is_relative: bool,
}

#[derive(Debug)]
pub struct LibraryLinkExtractor {
    blend_file_path: PathBuf,
}

impl LibraryLinkExtractor {
    pub fn new<P: AsRef<Path>>(blend_file_path: P) -> Self {
        Self {
            blend_file_path: blend_file_path.as_ref().to_path_buf(),
        }
    }

    pub fn extract_links(&self, blocks: &[Block], dna: &Dna) -> Result<Vec<LibraryLink>> {
        let mut links = Vec::new();

        // Extract library links (LI blocks)
        links.extend(self.extract_library_blocks(blocks, dna)?);

        // Extract image links (IM blocks)
        links.extend(self.extract_image_blocks(blocks, dna)?);

        // Extract sound links (SO blocks)
        links.extend(self.extract_sound_blocks(blocks, dna)?);

        // Extract movie clip links (MC blocks)
        links.extend(self.extract_movie_clip_blocks(blocks, dna)?);

        Ok(links)
    }

    fn extract_library_blocks(&self, blocks: &[Block], dna: &Dna) -> Result<Vec<LibraryLink>> {
        let mut links = Vec::new();

        for block in blocks {
            if &block.code[..2] == b"LI" {
                if let Some(link) = self.parse_library_block(block, dna)? {
                    links.push(link);
                }
            }
        }

        Ok(links)
    }

    fn extract_image_blocks(&self, blocks: &[Block], dna: &Dna) -> Result<Vec<LibraryLink>> {
        let mut links = Vec::new();

        for block in blocks {
            if &block.code[..2] == b"IM" {
                if let Some(link) = self.parse_image_block(block, dna)? {
                    links.push(link);
                }
            }
        }

        Ok(links)
    }

    fn extract_sound_blocks(&self, blocks: &[Block], dna: &Dna) -> Result<Vec<LibraryLink>> {
        let mut links = Vec::new();

        for block in blocks {
            if &block.code[..2] == b"SO" {
                if let Some(link) = self.parse_sound_block(block, dna)? {
                    links.push(link);
                }
            }
        }

        Ok(links)
    }

    fn extract_movie_clip_blocks(&self, blocks: &[Block], dna: &Dna) -> Result<Vec<LibraryLink>> {
        let mut links = Vec::new();

        for block in blocks {
            if &block.code[..2] == b"MC" {
                if let Some(link) = self.parse_movie_clip_block(block, dna)? {
                    links.push(link);
                }
            }
        }

        Ok(links)
    }

    fn parse_library_block(&self, block: &Block, dna: &Dna) -> Result<Option<LibraryLink>> {
        // Library blocks contain Library structures
        // The path is typically in the 'filepath' field
        let path = self.extract_string_field(block, dna, "filepath")?;

        if let Some(path_str) = path {
            if !path_str.is_empty() {
                // Blender uses "//" prefix for relative paths
                let is_relative = path_str.starts_with("//") || !path_str.starts_with('/');
                Ok(Some(LibraryLink {
                    path: path_str,
                    absolute_path: None,
                    block_type: "Library".to_string(),
                    block_name: self.extract_string_field(block, dna, "name")?,
                    is_relative,
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn parse_image_block(&self, block: &Block, dna: &Dna) -> Result<Option<LibraryLink>> {
        // Image blocks contain Image structures
        // The path is typically in the 'filepath' field
        let path = self.extract_string_field(block, dna, "filepath")?;

        if let Some(path_str) = path {
            if !path_str.is_empty() {
                // Blender uses "//" prefix for relative paths
                let is_relative = path_str.starts_with("//") || !path_str.starts_with('/');
                Ok(Some(LibraryLink {
                    path: path_str,
                    absolute_path: None,
                    block_type: "Image".to_string(),
                    block_name: self.extract_string_field(block, dna, "name")?,
                    is_relative,
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn parse_sound_block(&self, block: &Block, dna: &Dna) -> Result<Option<LibraryLink>> {
        // Sound blocks contain bSound structures
        // The path is typically in the 'filepath' field
        let path = self.extract_string_field(block, dna, "filepath")?;

        if let Some(path_str) = path {
            if !path_str.is_empty() {
                // Blender uses "//" prefix for relative paths
                let is_relative = path_str.starts_with("//") || !path_str.starts_with('/');
                Ok(Some(LibraryLink {
                    path: path_str,
                    absolute_path: None,
                    block_type: "Sound".to_string(),
                    block_name: self.extract_string_field(block, dna, "name")?,
                    is_relative,
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn parse_movie_clip_block(&self, block: &Block, dna: &Dna) -> Result<Option<LibraryLink>> {
        // Movie clip blocks contain MovieClip structures
        // The path is typically in the 'filepath' field
        let path = self.extract_string_field(block, dna, "filepath")?;

        if let Some(path_str) = path {
            if !path_str.is_empty() {
                // Blender uses "//" prefix for relative paths
                let is_relative = path_str.starts_with("//") || !path_str.starts_with('/');
                Ok(Some(LibraryLink {
                    path: path_str,
                    absolute_path: None,
                    block_type: "MovieClip".to_string(),
                    block_name: self.extract_string_field(block, dna, "name")?,
                    is_relative,
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn extract_string_field(
        &self,
        block: &Block,
        _dna: &Dna,
        field_name: &str,
    ) -> Result<Option<String>> {
        if block.data.is_empty() {
            return Ok(None);
        }

        // Based on debug analysis of actual .blend files:
        // - Library blocks (LI): filepath at offset 144 (0x90), name at 32
        // - Image blocks (IM): filepath at offset 104 (0x68), name at 0
        // - Sound blocks (SO): filepath at offset 104 (0x68), name at 0
        // - Movie clip blocks (MC): filepath at offset 104 (0x68), name at 0

        let (offset, max_len) = match field_name {
            "filepath" => {
                if block.code.starts_with(b"LI") {
                    (144, 1024) // Library filepath can be long
                } else if block.code.starts_with(b"IM")
                    || block.code.starts_with(b"SO")
                    || block.code.starts_with(b"MC")
                {
                    (104, 1024) // Common offset for other asset types
                } else {
                    return Ok(None);
                }
            }
            "name" => {
                if block.code.starts_with(b"LI") {
                    (32, 64) // Library name
                } else if block.code.starts_with(b"IM")
                    || block.code.starts_with(b"SO")
                    || block.code.starts_with(b"MC")
                {
                    (0, 64) // Name at start for other types
                } else {
                    return Ok(None);
                }
            }
            _ => return Ok(None),
        };

        if offset >= block.data.len() {
            return Ok(None);
        }

        // Find null-terminated string starting at offset
        let start = offset;
        let search_end = (start + max_len).min(block.data.len());
        let end = block.data[start..search_end]
            .iter()
            .position(|&b| b == 0)
            .map(|pos| start + pos)
            .unwrap_or(search_end);

        if start >= end {
            return Ok(None);
        }

        let string_bytes = &block.data[start..end];

        // Filter out non-printable characters and control codes
        let filtered: Vec<u8> = string_bytes
            .iter()
            .copied()
            .filter(|&b| (32..=126).contains(&b))
            .collect();

        if filtered.is_empty() {
            return Ok(None);
        }

        match String::from_utf8(filtered) {
            Ok(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() || trimmed.len() < 3 {
                    Ok(None)
                } else {
                    Ok(Some(trimmed.to_string()))
                }
            }
            Err(_) => Ok(None),
        }
    }

    pub fn resolve_relative_paths(&self, links: &mut Vec<LibraryLink>) -> Result<()> {
        let blend_dir = self
            .blend_file_path
            .parent()
            .ok_or_else(|| BlendFileError::InvalidFormat("Invalid blend file path".to_string()))?;

        for link in links {
            if link.is_relative {
                let resolved_path = blend_dir.join(&link.path);
                link.absolute_path = Some(resolved_path.to_string_lossy().into_owned());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_link_extractor_new() {
        let extractor = LibraryLinkExtractor::new("/path/to/file.blend");
        assert_eq!(
            extractor.blend_file_path.to_string_lossy(),
            "/path/to/file.blend"
        );
    }

    #[test]
    fn test_resolve_relative_paths() {
        let mut links = vec![
            LibraryLink {
                path: "textures/wood.jpg".to_string(),
                absolute_path: None,
                block_type: "Image".to_string(),
                block_name: None,
                is_relative: true,
            },
            LibraryLink {
                path: "/absolute/path/file.blend".to_string(),
                absolute_path: None,
                block_type: "Library".to_string(),
                block_name: None,
                is_relative: false,
            },
        ];

        let extractor = LibraryLinkExtractor::new("/home/user/project/scene.blend");
        extractor.resolve_relative_paths(&mut links).unwrap();

        assert_eq!(
            links[0].absolute_path,
            Some("/home/user/project/textures/wood.jpg".to_string())
        );
        assert_eq!(links[1].absolute_path, None);
    }
}
