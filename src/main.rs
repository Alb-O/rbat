use blend_file_reader::BlendFile;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "blend-file-reader")]
#[command(about = "A tool to read and analyze Blender .blend files")]
#[command(version = "1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all library links in a blend file
    Links {
        /// Path to the blend file
        #[arg(short, long)]
        file: PathBuf,

        /// Output format (json, table)
        #[arg(short = 'o', long, default_value = "table")]
        format: String,

        /// Include absolute paths
        #[arg(short, long)]
        absolute: bool,
    },

    /// List all blocks in a blend file
    Blocks {
        /// Path to the blend file
        #[arg(short, long)]
        file: PathBuf,

        /// Filter by block type
        #[arg(short = 't', long)]
        filter: Option<String>,
    },

    /// Show file summary
    Summary {
        /// Path to the blend file
        #[arg(short, long)]
        file: PathBuf,
    },

    /// Debug library blocks
    Debug {
        /// Path to the blend file
        #[arg(short, long)]
        file: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Links {
            file,
            format,
            absolute,
        } => {
            let blend_file = BlendFile::open(&file)?;
            let links = blend_file.get_library_links()?;

            if links.is_empty() {
                println!("No library links found in {file}", file = file.display());
                return Ok(());
            }

            match format.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&links)?;
                    println!("{json}");
                }
                "table" => {
                    println!("Library links in {file}:", file = file.display());
                    println!(
                        "{:<15} {:<30} {:<50} {:<10}",
                        "Type", "Name", "Path", "Relative"
                    );
                    println!("{:-<15} {:-<30} {:-<50} {:-<10}", "", "", "", "");

                    for link in links {
                        let name = link.block_name.unwrap_or_else(|| "N/A".to_string());
                        let path = if absolute && link.absolute_path.is_some() {
                            link.absolute_path.unwrap()
                        } else {
                            link.path.clone()
                        };

                        println!(
                            "{:<15} {:<30} {:<50} {:<10}",
                            link.block_type,
                            name,
                            path,
                            if link.is_relative { "Yes" } else { "No" }
                        );
                    }
                }
                _ => {}
            }
        }

        Commands::Blocks { file, filter } => {
            let blend_file = BlendFile::open(&file)?;

            let blocks = match filter {
                Some(ref filter_type) => match filter_type.as_str() {
                    "library" => blend_file.get_library_blocks(),
                    "image" => blend_file.get_image_blocks(),
                    "sound" => blend_file.get_sound_blocks(),
                    "movieclip" => blend_file.get_movie_clip_blocks(),
                    "mesh" => blend_file.get_mesh_blocks(),
                    "material" => blend_file.get_material_blocks(),
                    "texture" => blend_file.get_texture_blocks(),
                    _ => {
                        eprintln!("Unknown block type: {filter_type}");
                        return Ok(());
                    }
                },
                None => blend_file.blocks.iter().collect(),
            };

            if blocks.is_empty() {
                println!("No blocks found");
                return Ok(());
            }

            println!("Blocks in {file}:", file = file.display());
            println!(
                "{:<8} {:<10} {:<15} {:<10}",
                "Code", "Size", "Address", "Count"
            );
            println!("{:-<8} {:-<10} {:-<15} {:-<10}", "", "", "", "");

            for block in blocks {
                println!(
                    "{:<8} {:<10} 0x{:<13x} {:<10}",
                    String::from_utf8_lossy(&block.code),
                    block.size,
                    block.old_memory_address,
                    block.count
                );
            }
        }

        Commands::Summary { file } => {
            let blend_file = BlendFile::open(&file)?;
            blend_file.print_summary();
        }

        Commands::Debug { file } => {
            use blend_file_reader::debug::debug_library_blocks;
            debug_library_blocks(&file)?;
        }
    }

    Ok(())
}
