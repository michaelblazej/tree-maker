# JSON Configuration Schema

This document describes the JSON configuration format that can be used with the `tree-maker` CLI tool to generate custom trees.

## Example Configuration

```json
{
    "seed": 12345,
    "type": "Deciduous",
    "bark": {
      "type": "Oak",
      "tint": 16777215,
      "flatShading": false,
      "textured": true,
      "textureScale": {
        "x": 1.0,
        "y": 1.0
      }
    },
    "branch": {
      "levels": 3,
      "angle": {
        "0": 30,
        "1": 25,
        "2": 20
      },
      "children": {
        "0": 3,
        "1": 2,
        "2": 1
      },
      "force": {
        "direction": {
          "x": 0,
          "y": 1,
          "z": 0
        },
        "strength": 0.5
      },
      "gnarliness": {
        "0": 0.2,
        "1": 0.3,
        "2": 0.4
      },
      "length": {
        "0": 5.0,
        "1": 3.5,
        "2": 2.0
      },
      "radius": {
        "0": 0.5,
        "1": 0.3,
        "2": 0.1
      },
      "sections": {
        "0": 5,
        "1": 4,
        "2": 3
      },
      "segments": 8,
      "start": {
        "0": 0.0,
        "1": 0.3,
        "2": 0.6
      },
      "taper": {
        "0": 0.8,
        "1": 0.7,
        "2": 0.6
      },
      "twist": {
        "0": 10,
        "1": 15,
        "2": 20
      }
    },
    "leaves": {
      "type": "Oak",
      "billboard": "Double",
      "angle": 45,
      "count": 500,
      "start": 0.7,
      "size": 0.2,
      "sizeVariance": 0.05,
      "tint": 65280,
      "alphaTest": 0.5
    }
}
```

## Schema Description

### Top-Level Fields

| Field | Type | Description |
|-------|------|-------------|
| `seed` | number (optional) | Random seed for tree generation |
| `type` | string | Type of tree (e.g., "Deciduous", "Pine", "Willow", "Palm") |
| `bark` | object | Configuration for tree bark |
| `branch` | object | Configuration for branch structure |
| `leaves` | object | Configuration for leaves |

### Bark Configuration

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | Type of bark (e.g., "Oak", "Pine") |
| `tint` | number | RGB color tint for the bark (hexadecimal) |
| `flatShading` | boolean | Whether to use flat shading |
| `textured` | boolean | Whether to apply texture to the bark |
| `textureScale` | object | Scale of the bark texture |
| `textureScale.x` | number | X-axis scale |
| `textureScale.y` | number | Y-axis scale |

### Branch Configuration

| Field | Type | Description |
|-------|------|-------------|
| `levels` | number | Number of branch levels (from trunk to smallest branches) |
| `angle` | object | Branch angles (in degrees) for each level |
| `children` | object | Number of child branches for each level |
| `force` | object | External force affecting branch growth |
| `gnarliness` | object | How twisted/gnarled the branches are at each level (0.0-1.0) |
| `length` | object | Branch length (in meters) for each level |
| `radius` | object | Branch radius (in meters) for each level |
| `sections` | object | Number of sections per branch for each level |
| `segments` | number | Number of segments per section (affects roundness) |
| `start` | object | Starting point along parent (0.0-1.0) for each level |
| `taper` | object | Branch tapering factor (0.0-1.0) for each level |
| `twist` | object | Branch twisting (in degrees) for each level |

#### Branch Level Objects

Each branch level object uses string keys representing level indices ("0", "1", "2", etc.). The trunk is level "0", its immediate children are level "1", and so on.

### Force Configuration

| Field | Type | Description |
|-------|------|-------------|
| `direction` | object | Direction of the force |
| `direction.x` | number | X component |
| `direction.y` | number | Y component |
| `direction.z` | number | Z component |
| `strength` | number | Strength of the force (0.0-1.0) |

### Leaves Configuration

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | Type of leaves (e.g., "Oak", "Pine") |
| `billboard` | string | Billboard mode ("Single", "Double") |
| `angle` | number | Leaf angle in degrees |
| `count` | number | Total number of leaves |
| `start` | number | Where leaves start on branches (0.0-1.0) |
| `size` | number | Size of leaves in meters |
| `sizeVariance` | number | Variance in leaf size (0.0-1.0) |
| `tint` | number | RGB color tint for leaves (hexadecimal) |
| `alphaTest` | number | Alpha test threshold (0.0-1.0) |

## Supported Tree Types

The `tree-maker` tool currently supports the following tree types:

- "Oak" or "Deciduous" - Broad-leaf deciduous tree with a bulbous canopy
- "Pine" or "Conifer" - Coniferous tree with conical layers
- "Willow" or "Weeping" - Willow tree with drooping branches
- "Palm" or "Tropical" - Palm tree with a tall trunk and radiating fronds

## Usage

```bash
# Generate a tree from a JSON configuration
cargo run -- from-json --config inputs/example.json --output custom_tree.glb
```

You can also use a JSON configuration file programmatically when using tree-maker as a library:

```rust
use tree_maker::config::{read_config_from_file, convert_json_config_to_tree_config};
use tree_maker::generate_tree;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read JSON config
    let json_config = read_config_from_file("inputs/example.json")?;
    
    // Convert to TreeConfig
    let tree_config = convert_json_config_to_tree_config(&json_config)?;
    
    // Generate the tree
    generate_tree(&tree_config, Path::new("output.glb"))?;
    
    Ok(())
}
```
