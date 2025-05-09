use clap::Parser;
use std::error::Error;
use std::path::PathBuf;

// Import from library interface
use tree_maker::tree::{generate_oak_tree, generate_pine_tree, generate_willow_tree, generate_palm_tree};
use tree_maker::config::{read_config_from_file, get_trunk_config};

/// A Rust library and CLI tool for generating 3D tree models
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the JSON configuration file
    #[arg(required = true)]
    config_file: PathBuf,
    
    /// Output file path (default: tree.glb)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Check if config file exists
    if !cli.config_file.exists() {
        return Err(format!("Config file not found: {}", cli.config_file.display()).into());
    }
    
    println!("Reading configuration from file: {}", cli.config_file.display());
    
    // Read and parse JSON configuration
    let json_config = read_config_from_file(&cli.config_file)?;
    
    // Get the branch configuration from JSON
    let branch_config = get_branch_config(&json_config);

    // Generate the tree
    generate_tree(branch_config, Some(123456), cli.output.as_deref())?;
    
    Ok(())
}
