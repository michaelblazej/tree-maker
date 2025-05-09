// Public modules
pub mod tree;
pub mod config;


/// Configuration for tree generation
#[derive(Debug, Clone)]
pub struct BranchConfig {
    pub length: f32,
    pub radius: f32,
    pub segments: u32,
    pub angle: f32,
    pub taper: f32,
    pub twist: f32,
    pub gnarliness: f32,
    pub children: u32,
    pub children_config: Option<Box<BranchConfig>>,
}