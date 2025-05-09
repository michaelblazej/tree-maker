use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use std::io;

// Import from library interface
use tree_maker::generate_tree;
use tree_maker::config::{read_config_from_file, convert_json_config_to_tree_config};

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
    
    // Convert JSON config to TreeConfig
    let tree_config = convert_json_config_to_tree_config(&json_config)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    // Validate the converted config
    if let Err(msg) = tree_maker::validate_config(&tree_config) {
        return Err(msg.into());
    }
    
    // Default output path if not specified
    let output_path = match cli.output {
        Some(path) => path,
        None => PathBuf::from("tree.glb"),
    };
    
    println!("Generating {} tree from JSON configuration", 
        tree_config.tree_type.as_str());
        
    // Generate tree using the library interface
    generate_tree(&tree_config, &output_path)?;
    
    println!("Tree generated successfully: {}", output_path.display());
    Ok(())
}
