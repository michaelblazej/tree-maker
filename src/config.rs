use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::BranchConfig;

/// JSON configuration for tree generation
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonTreeConfig {
    /// Random seed for generation
    pub seed: Option<u64>,
    /// Type of tree (e.g., "Deciduous", "Pine", etc.)
    #[serde(rename = "type")]
    pub tree_type: String,
    /// Bark configuration
    pub bark: BarkConfig,
    /// Trunk configuration (root branch)
    pub trunk: JsonBranchConfig,
}

/// Bark configuration
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BarkConfig {
    /// Type of bark (e.g., "Oak", "Pine", etc.)
    #[serde(rename = "type")]
    pub bark_type: String,
    /// Color tint (RGB)
    pub tint: u32,
    /// Whether to use flat shading
    pub flat_shading: bool,
    /// Whether the bark is textured
    pub textured: bool,
    /// Texture scale
    pub texture_scale: TextureScale,
}

/// Texture scale
#[derive(Debug, Serialize, Deserialize)]
pub struct TextureScale {
    /// X scale
    pub x: f32,
    /// Y scale
    pub y: f32,
}

/// JSON Branch configuration for the hierarchical branch structure
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonBranchConfig {
    /// Length of the branch
    pub length: f32,
    /// Radius of the branch
    pub radius: f32,
    /// Number of segments around the branch circumference
    pub segments: u32,
    /// Angle of the branch relative to parent
    pub angle: f32,
    /// Tapering factor of the branch (how much it narrows toward the tip)
    pub taper: f32,
    /// Twist amount along the branch axis
    pub twist: f32,
    /// Gnarliness factor (randomness in branch shape)
    pub gnarliness: f32,
    /// Number of child branches
    pub children: u32,
    /// Configuration for child branches
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "children_config")]
    pub children_config: Option<Box<JsonBranchConfig>>,
}

/// Force direction (used in the bark texture orientation)
#[derive(Debug, Serialize, Deserialize)]
pub struct ForceDirection {
    /// X component
    pub x: f32,
    /// Y component
    pub y: f32,
    /// Z component
    pub z: f32,
}

/// Read a tree configuration from a JSON file
pub fn read_config_from_file<P: AsRef<Path>>(path: P) -> Result<JsonTreeConfig, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

/// Convert a JsonBranchConfig to the application's BranchConfig
pub fn convert_json_branch_to_branch_config(json_branch: &JsonBranchConfig) -> BranchConfig {
    println!("Converting JsonBranchConfig: children={}, has_children_config={}", 
              json_branch.children, json_branch.children_config.is_some());
    // Recursively convert the children configuration if it exists
    let children_config = json_branch.children_config
        .as_ref()
        .map(|config| {
            println!("  Found child config with radius={}", config.radius);
            Box::new(convert_json_branch_to_branch_config(config))
        });
        
    println!("  Resulting children_config is {}", if children_config.is_some() { "Some" } else { "None" });
    
    BranchConfig {
        length: json_branch.length,
        radius: json_branch.radius,
        segments: json_branch.segments,
        angle: json_branch.angle,
        taper: json_branch.taper,
        twist: json_branch.twist,
        gnarliness: json_branch.gnarliness,
        children: json_branch.children,
        children_config,
    }
}

/// Get the trunk configuration from the JSON config
pub fn get_branch_config(json_config: &JsonTreeConfig) -> BranchConfig {
    convert_json_branch_to_branch_config(&json_config.trunk)
}
