// Public modules
pub mod tree;
pub mod config;

// Re-export the main functionality for easier library usage
pub use tree::{
    generate_oak_tree,
    generate_pine_tree,
    generate_willow_tree,
    generate_palm_tree,
};

/// Supported tree types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TreeType {
    /// Oak tree with a bulbous canopy
    Oak,
    /// Pine tree with conical layers
    Pine,
    /// Willow tree with drooping branches
    Willow,
    /// Palm tree with a tall trunk and radiating fronds
    Palm,
}

impl TreeType {
    /// Converts a string to a TreeType, if valid
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "oak" => Some(TreeType::Oak),
            "pine" => Some(TreeType::Pine),
            "willow" => Some(TreeType::Willow),
            "palm" => Some(TreeType::Palm),
            _ => None,
        }
    }
    
    /// Returns the string representation of this tree type
    pub fn as_str(&self) -> &'static str {
        match self {
            TreeType::Oak => "oak",
            TreeType::Pine => "pine",
            TreeType::Willow => "willow",
            TreeType::Palm => "palm",
        }
    }
}

/// Configuration for tree generation
#[derive(Debug, Clone)]
pub struct TreeConfig {
    /// Type of tree to generate
    pub tree_type: TreeType,
    /// Height of the tree in meters
    pub height: f32,
    /// Density of branches (0.0 to 1.0)
    pub branch_density: f32,
    /// Level of detail (1-5, higher is more detailed)
    pub detail_level: u32,
    /// Random seed for generation
    pub seed: Option<u64>,
}

impl Default for TreeConfig {
    fn default() -> Self {
        Self {
            tree_type: TreeType::Oak,
            height: 5.0,
            branch_density: 0.5,
            detail_level: 3,
            seed: None,
        }
    }
}

/// Validates the TreeConfig parameters
/// 
/// # Returns
/// 
/// Ok(()) if the configuration is valid, or an error message if not
pub fn validate_config(config: &TreeConfig) -> Result<(), String> {
    if config.branch_density < 0.0 || config.branch_density > 1.0 {
        return Err("Branch density must be between 0.0 and 1.0".to_string());
    }

    if config.detail_level < 1 || config.detail_level > 5 {
        return Err("Detail level must be between 1 and 5".to_string());
    }

    Ok(())
}

/// Generate a tree based on the provided configuration
/// 
/// # Arguments
/// 
/// * `config` - The tree configuration
/// * `output_path` - Path where the 3D model will be saved
/// 
/// # Returns
/// 
/// Result indicating success or an error
pub fn generate_tree(config: &TreeConfig, output_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    // Validate config
    validate_config(config)?;
    
    // Generate tree based on type
    match config.tree_type {
        TreeType::Pine => tree::generate_pine_tree(
            config.height, 
            config.branch_density, 
            config.detail_level, 
            config.seed, 
            output_path
        ),
        TreeType::Oak => tree::generate_oak_tree(
            config.height, 
            config.branch_density, 
            config.detail_level, 
            config.seed, 
            output_path
        ),
        TreeType::Willow => tree::generate_willow_tree(
            config.height, 
            config.branch_density, 
            config.detail_level, 
            config.seed, 
            output_path
        ),
        TreeType::Palm => tree::generate_palm_tree(
            config.height, 
            config.branch_density, 
            config.detail_level, 
            config.seed, 
            output_path
        ),
    }
}
