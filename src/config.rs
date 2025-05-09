use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::{TreeConfig, TreeType};

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
    /// Branch configuration
    pub branch: BranchConfig,
    /// Leaves configuration
    pub leaves: LeavesConfig,
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

/// Branch configuration
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchConfig {
    /// Number of branch levels
    pub levels: u32,
    /// Branch angles for each level
    pub angle: HashMap<String, u32>,
    /// Number of children for each level
    pub children: HashMap<String, u32>,
    /// Force direction and strength
    pub force: ForceConfig,
    /// Branch gnarliness for each level
    pub gnarliness: HashMap<String, f32>,
    /// Branch length for each level
    pub length: HashMap<String, f32>,
    /// Branch radius for each level
    pub radius: HashMap<String, f32>,
    /// Sections per branch for each level
    pub sections: HashMap<String, u32>,
    /// Segments per branch section
    pub segments: u32,
    /// Branch start point for each level
    pub start: HashMap<String, f32>,
    /// Branch tapering for each level
    pub taper: HashMap<String, f32>,
    /// Branch twist for each level
    pub twist: HashMap<String, u32>,
}

/// Force configuration
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceConfig {
    /// Force direction
    pub direction: ForceDirection,
    /// Force strength
    pub strength: f32,
}

/// Force direction
#[derive(Debug, Serialize, Deserialize)]
pub struct ForceDirection {
    /// X component
    pub x: f32,
    /// Y component
    pub y: f32,
    /// Z component
    pub z: f32,
}

/// Leaves configuration
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeavesConfig {
    /// Type of leaves (e.g., "Oak", "Pine", etc.)
    #[serde(rename = "type")]
    pub leaf_type: String,
    /// Billboard mode
    pub billboard: String,
    /// Leaf angle
    pub angle: u32,
    /// Number of leaves
    pub count: u32,
    /// Where leaves start on branches (0.0 to 1.0)
    pub start: f32,
    /// Leaf size
    pub size: f32,
    /// Variance in leaf size
    pub size_variance: f32,
    /// Color tint (RGB)
    pub tint: u32,
    /// Alpha test value
    pub alpha_test: f32,
}

/// Read a tree configuration from a JSON file
pub fn read_config_from_file<P: AsRef<Path>>(path: P) -> Result<JsonTreeConfig, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

/// Convert a JSON tree configuration to a TreeConfig
pub fn convert_json_config_to_tree_config(json_config: &JsonTreeConfig) -> Result<TreeConfig, String> {
    // Map the JSON tree type to our TreeType enum
    let tree_type = match json_config.tree_type.to_lowercase().as_str() {
        "deciduous" | "oak" => TreeType::Oak,
        "conifer" | "pine" => TreeType::Pine,
        "weeping" | "willow" => TreeType::Willow,
        "tropical" | "palm" => TreeType::Palm,
        _ => return Err(format!("Unknown tree type: {}", json_config.tree_type)),
    };
    
    // Derive branch density from parameters
    let branch_density = json_config.branch.children
        .values()
        .map(|&v| v as f32)
        .sum::<f32>() / (json_config.branch.levels as f32 * 3.0); // Normalize to 0-1 range
    let branch_density = branch_density.min(1.0).max(0.0);
    
    // Derive height from the branch length
    let height = json_config.branch.length
        .values()
        .map(|&v| v)
        .sum::<f32>();
    
    // Derive detail level from segments and sections
    let avg_sections = json_config.branch.sections
        .values()
        .map(|&v| v as f32)
        .sum::<f32>() / json_config.branch.levels as f32;
    
    let detail_level = ((json_config.branch.segments as f32 * avg_sections) / 10.0)
        .min(5.0)
        .max(1.0) as u32;
    
    Ok(TreeConfig {
        tree_type,
        height,
        branch_density,
        detail_level,
        seed: json_config.seed,
    })
}
