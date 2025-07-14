# Blend File Reader

A Rust library and CLI tool for reading Blender `.blend` files and extracting library link information without requiring Blender to be installed.

## Overview

This project provides a complete implementation for parsing Blender's binary `.blend` file format, allowing you to:
- Read `.blend` files directly without Blender
- Extract library links (external file references)
- Analyze file structure and metadata
- Filter blocks by type (Library, Image, Sound, etc.)
- Resolve relative paths to absolute paths

## Features

- **Complete file format parsing**: Handles Blender's binary format including headers, DNA structures, and data blocks
- **Library link extraction**: Identifies external file references from Library, Image, Sound, and MovieClip blocks
- **Path resolution**: Converts relative paths to absolute paths based on blend file location
- **CLI interface**: Command-line tool for quick analysis
- **Comprehensive error handling**: Detailed error messages for debugging
- **Memory efficient**: Uses memory mapping for large files
- **Cross-platform**: Works on Windows, macOS, and Linux

## Installation

### From Source

```bash
git clone <repository-url>
cd blend-file-reader
cargo build --release
```

The binary will be available at `target/release/blend-file-reader`.

## Usage

### Command Line Interface

#### List library links
```bash
# Basic usage
blend-file-reader links --file scene.blend

# JSON output
blend-file-reader links --file scene.blend --format json

# Include absolute paths
blend-file-reader links --file scene.blend --absolute
```

#### List blocks
```bash
# All blocks
blend-file-reader blocks --file scene.blend

# Filter by type
blend-file-reader blocks --file scene.blend --filter image
```

#### File summary
```bash
blend-file-reader summary --file scene.blend
```

### Library Usage

Add to your `Cargo.toml`:
```toml
[dependencies]
blend-file-reader = "0.1.0"
```

Basic usage:
```rust
use blend_file_reader::BlendFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let blend_file = BlendFile::open("scene.blend")?;
    
    // Get all library links
    let links = blend_file.get_library_links()?;
    
    for link in links {
        println!("Type: {}, Path: {}", link.block_type, link.path);
        if let Some(abs_path) = link.absolute_path {
            println!("  Absolute: {}", abs_path);
        }
    }
    
    Ok(())
}
```

## File Format Support

### Supported Block Types
- **LI**: Library blocks (external .blend file references)
- **IM**: Image blocks (texture/image file references)
- **SO**: Sound blocks (audio file references)
- **MC**: MovieClip blocks (video file references)
- **ME**: Mesh blocks
- **MA**: Material blocks
- **TE**: Texture blocks

### Blender Versions
- Supports Blender 2.79 and later
- Handles both 32-bit and 64-bit pointer sizes
- Supports little-endian and big-endian formats

## Architecture

### Core Components

- **Header**: Parses file format version, pointer size, and endianness
- **DNA**: Structure definitions for Blender's internal data types
- **Blocks**: Individual data segments containing Blender objects
- **LibraryLink**: Extracted external file references with metadata

### Error Handling

The library uses custom error types for clear debugging:
- `IoError`: File I/O issues
- `InvalidFormat`: Malformed blend files
- `UnsupportedVersion`: Incompatible Blender versions
- `ParseError`: Data parsing failures

## Development

### Running Tests
```bash
cargo test
```

### Building
```bash
cargo build --release
```

### Examples
```bash
# Run the demo
cargo run --example demo

# CLI usage
cargo run -- links --file path/to/file.blend
```

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

MIT License - see LICENSE file for details.

## Acknowledgments

This project is inspired by the original Python `blender-asset-tracer` library and the Blender file format documentation.