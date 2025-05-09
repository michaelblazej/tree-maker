# Tree-Maker

A Rust library and CLI tool for procedurally generating 3D tree models and exporting them to the glTF/GLB format.

## Features

- Procedural generation of realistic 3D tree models
- Customizable parameters for different tree species and styles
- Export to glTF/GLB formats for use in 3D applications
- Use as a library in your Rust projects
- Command-line interface for generating trees with different parameters

## Installation

```bash
# Clone the repository
git clone https://github.com/michaelblazej/tree-maker.git
cd tree-maker

# Build the project
cargo build --release
```

## Usage

### As a CLI Tool

```bash
# Generate a tree using a JSON configuration file
cargo run -- inputs/example.json

# Generate a tree with a custom output path
cargo run -- inputs/example.json --output custom_tree.glb

# Show help
cargo run -- --help
```

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
tree-maker = "0.1.0"
```

Example usage in your Rust code:

```rust
use std::path::PathBuf;
use tree_maker::{TreeConfig, TreeType, generate_tree};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a tree configuration
    let config = TreeConfig {
        tree_type: TreeType::Pine,
        height: 8.0,
        branch_density: 0.7,
        detail_level: 4,
        seed: Some(42), // For reproducible results
    };
    
    // Generate the tree model
    let output_path = PathBuf::from("my_pine_tree.glb");
    generate_tree(&config, &output_path)?;
    
    println!("Tree generated at: {}", output_path.display());
    Ok(())
}
```

## Dependencies

- [mesh-tools](https://github.com/michaelblazej/mesh-tools): A Rust library for creating and exporting 3D models to glTF/GLB format
- Other standard Rust libraries for math operations, random number generation, etc.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
