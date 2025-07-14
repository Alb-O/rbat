use blend_file_reader::BlendFile;
use std::fs;

#[test]
fn test_blend_file_header_parsing() {
    // Create a minimal valid blend file header for testing
    let test_dir = "tests/test_blend_files";
    fs::create_dir_all(test_dir).unwrap();

    let test_file = format!("{test_dir}/header_test.blend");

    // Create a minimal blend file with valid header
    let header_data = b"BLENDER-v279RENDH";
    fs::write(&test_file, header_data).unwrap();

    let result = BlendFile::open(&test_file);
    assert!(result.is_err()); // Should fail due to incomplete file

    // Clean up
    fs::remove_file(&test_file).unwrap();
}

#[test]
fn test_blend_file_error_handling() {
    let test_dir = "tests/test_blend_files";
    fs::create_dir_all(test_dir).unwrap();

    // Test with invalid file
    let invalid_file = format!("{test_dir}/invalid.blend");
    fs::write(&invalid_file, b"INVALID FILE").unwrap();

    let result = BlendFile::open(&invalid_file);
    assert!(result.is_err());

    // Clean up
    fs::remove_file(&invalid_file).unwrap();
}

#[test]
fn test_library_link_extraction() {
    // This test will be more comprehensive once we have proper test files
    let test_dir = "tests/test_blend_files";
    fs::create_dir_all(test_dir).unwrap();

    // For now, test the structure
    let blend_file = BlendFile {
        path: std::path::PathBuf::from("test.blend"),
        header: blend_file_reader::header::Header {
            magic: *b"BLENDER",
            pointer_size: blend_file_reader::header::PointerSize::Bits64,
            endianness: blend_file_reader::header::Endianness::Little,
            version: 279,
        },
        dna: blend_file_reader::dna::Dna {
            structs: std::collections::HashMap::new(),
            type_sizes: std::collections::HashMap::new(),
        },
        blocks: vec![],
    };

    let links = blend_file.get_library_links();
    assert!(links.is_ok());
    assert!(links.unwrap().is_empty());
}

#[test]
fn test_block_filtering() {
    use blend_file_reader::block::Block;

    let blend_file = BlendFile {
        path: std::path::PathBuf::from("test.blend"),
        header: blend_file_reader::header::Header {
            magic: *b"BLENDER",
            pointer_size: blend_file_reader::header::PointerSize::Bits64,
            endianness: blend_file_reader::header::Endianness::Little,
            version: 279,
        },
        dna: blend_file_reader::dna::Dna {
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
            Block {
                code: *b"SO\0\0",
                size: 150,
                old_memory_address: 0x3000,
                sdna_index: 2,
                count: 1,
                data_offset: 500,
                data: vec![0; 150],
            },
        ],
    };

    let library_blocks = blend_file.get_library_blocks();
    assert_eq!(library_blocks.len(), 1);
    assert_eq!(&library_blocks[0].code[..2], b"LI");

    let image_blocks = blend_file.get_image_blocks();
    assert_eq!(image_blocks.len(), 1);
    assert_eq!(&image_blocks[0].code[..2], b"IM");

    let sound_blocks = blend_file.get_sound_blocks();
    assert_eq!(sound_blocks.len(), 1);
    assert_eq!(&sound_blocks[0].code[..2], b"SO");
}

#[test]
fn test_path_resolution() {
    use blend_file_reader::library_link::{LibraryLink, LibraryLinkExtractor};

    let extractor = LibraryLinkExtractor::new("/home/user/project/scene.blend");

    let mut links = vec![
        LibraryLink {
            path: "textures/wood.jpg".to_string(),
            absolute_path: None,
            block_type: "Image".to_string(),
            block_name: None,
            is_relative: true,
        },
        LibraryLink {
            path: "/absolute/path/texture.jpg".to_string(),
            absolute_path: None,
            block_type: "Image".to_string(),
            block_name: None,
            is_relative: false,
        },
    ];

    extractor.resolve_relative_paths(&mut links).unwrap();

    assert_eq!(
        links[0].absolute_path,
        Some("/home/user/project/textures/wood.jpg".to_string())
    );
    assert_eq!(links[1].absolute_path, None);
}

#[test]
fn test_cli_functionality() {
    use std::process::Command;

    // Test that the CLI binary can be built
    let status = Command::new("cargo")
        .args(["build", "--bin", "blend-file-reader"])
        .status()
        .unwrap();

    assert!(status.success());
}
