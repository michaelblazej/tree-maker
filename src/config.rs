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
    /// Radius at the start of the branch
    #[serde(rename = "startRadius")]
    pub start_radius: f32,
    /// Radius at the end of the branch
    #[serde(rename = "endRadius")]
    pub end_radius: f32,
    /// Number of segments along the branch length
    #[serde(rename = "lengthSegments")]
    pub length_segments: u32,
    /// Number of segments around the branch circumference
    #[serde(rename = "radialSegments")]
    pub radial_segments: u32,
    /// Backward compatibility field, will be ignored if length_segments is present
    #[serde(default)]
    pub segments: u32,
    /// Angle of the branch relative to parent
    pub angle: f32,
    /// Backward compatibility: Tapering factor of the branch
    #[serde(default)]
    pub taper: f32,
    /// Twist amount along the branch axis
    pub twist: f32,
    /// Gnarliness factor (randomness in branch shape)
    pub gnarliness: f32,
    /// Minimum rotation angle (degrees) for branch variations
    #[serde(rename = "minRotation", default = "default_min_rotation")]
    pub min_rotation: f32,
    /// Maximum rotation angle (degrees) for branch variations
    #[serde(rename = "maxRotation", default = "default_max_rotation")]
    pub max_rotation: f32,
    /// Minimum percentage position along parent branch where child branches can appear (0-100)
    #[serde(rename = "minBranchPosPct", default = "default_min_branch_pos_pct")]
    pub min_branch_pos_pct: f32,
    /// Maximum percentage position along parent branch where child branches can appear (0-100)
    #[serde(rename = "maxBranchPosPct", default = "default_max_branch_pos_pct")]
    pub max_branch_pos_pct: f32,
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
            println!("  Found child config with start_radius={}", config.start_radius);
            Box::new(convert_json_branch_to_branch_config(config))
        });
        
    println!("  Resulting children_config is {}", if children_config.is_some() { "Some" } else { "None" });
    
    // Determine segment count from length_segments or segments (backward compatibility)
    let segments = if json_branch.length_segments > 0 {
        json_branch.length_segments
    } else {
        json_branch.segments
    };

    BranchConfig {
        length: json_branch.length,
        start_radius: json_branch.start_radius,
        end_radius: json_branch.end_radius,
        length_segments: segments,
        radial_segments: json_branch.radial_segments,
        angle: json_branch.angle,
        twist: json_branch.twist,
        gnarliness: json_branch.gnarliness,
        min_rotation: json_branch.min_rotation,
        max_rotation: json_branch.max_rotation,
        min_branch_pos_pct: json_branch.min_branch_pos_pct,
        max_branch_pos_pct: json_branch.max_branch_pos_pct,
        children: json_branch.children,
        children_config,
    }
}

/// Get the trunk configuration from the JSON config
pub fn get_branch_config(json_config: &JsonTreeConfig) -> BranchConfig {
    convert_json_branch_to_branch_config(&json_config.trunk)
}

/// Default value for minimum rotation (20 degrees)
fn default_min_rotation() -> f32 {
    20.0
}

/// Default value for maximum rotation (40 degrees)
fn default_max_rotation() -> f32 {
    40.0
}

fn default_min_branch_pos_pct() -> f32 {
    10.0 // Default to 10% from start of branch
}

fn default_max_branch_pos_pct() -> f32 {
    90.0 // Default to 90% from start of branch
}
