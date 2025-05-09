use mesh_tools::{GltfBuilder, material, Triangle};
use nalgebra::{Point3, Vector3};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::error::Error;
use std::path::Path;
use std::f32::consts::PI;

// Common tree generation logic
struct TreeGenerator {
    rng: ChaCha8Rng,
    builder: GltfBuilder,
}

impl TreeGenerator {
    fn new(seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(s) => ChaCha8Rng::seed_from_u64(s),
            None => ChaCha8Rng::from_entropy(),
        };

        Self {
            rng,
            builder: GltfBuilder::new(),
        }
    }

    fn random_f32(&mut self, min: f32, max: f32) -> f32 {
        self.rng.gen_range(min..=max)
    }

    fn export(&mut self, output_path: &Path) -> Result<(), Box<dyn Error>> {
        self.builder.export_glb(output_path.to_str().unwrap())?;
        Ok(())
    }

    fn create_trunk_material(&mut self) -> usize {
        self.builder.create_basic_material(
            Some("Trunk".to_string().into()),
            [0.55, 0.27, 0.07, 1.0], // Brown
        )
    }

    fn create_leaf_material(&mut self, color: [f32; 4]) -> usize {
        self.builder.create_basic_material(
            Some("Leaves".to_string()),
            color,
        )
    }
    
 
}

/// Creates a custom branch mesh with noise for more natural looking branches
/// 
/// # Arguments
/// 
/// * `start_radius` - Radius at the base of the branch
/// * `end_radius` - Radius at the tip of the branch
/// * `height` - Length of the branch
/// * `height_segments` - Number of segments along the branch height
/// * `radial_segments` - Number of segments around the branch circumference
/// * `noise_level` - Amount of random variation (0.0-1.0) to apply to the vertices
/// 
/// # Returns
/// 
/// Tuple containing (vertices, indices) where vertices is a Vec<Point3<f32>> and indices is a Vec<Triangle>
pub fn branch_maker(start_radius: f32, end_radius: f32, height: f32, height_segments: u32, radial_segments: u32, noise_level: f32) -> (Vec<Point3<f32>>, Vec<Triangle>) {
    let radial_segments = radial_segments.max(3); // Minimum 3 segments
    let noise_level = noise_level.max(0.0).min(1.0); // Clamp noise level between 0 and 1
    
    let sections = height_segments; // Number of height sections
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    // Generate vertices
    for section in 0..=sections {
        let section_t = section as f32 / sections as f32;
        let section_radius = start_radius * (1.0 - section_t) + end_radius * section_t;
        let section_z = height * section_t;
        
        for segment in 0..radial_segments {
            let angle = 2.0 * PI * (segment as f32 / radial_segments as f32);
            
            // Add noise to x and z coordinates
            let noise_x = if noise_level > 0.0 { self.random_f32(-1.0, 1.0) * noise_level * section_radius * 0.3 } else { 0.0 };
            let noise_y = if noise_level > 0.0 { self.random_f32(-1.0, 1.0) * noise_level * section_radius * 0.3 } else { 0.0 };
            
            // Also add some minor noise to y to make it less uniform
            let noise_z = if noise_level > 0.0 { self.random_f32(-1.0, 1.0) * noise_level * height * 0.05 } else { 0.0 };
            
            let x = angle.cos() * section_radius + noise_x;
            let y = angle.sin() * section_radius + noise_y;
            let z = section_z + noise_z;
            
            vertices.push(Point3::new(x, y, z));
        }
    }
    
    // Generate indices for triangles
    for section in 0..sections {
        let section_start = section * radial_segments;
        let next_section_start = (section + 1) * radial_segments;
        
        for segment in 0..radial_segments {
            let current = section_start + segment;
            let next = section_start + (segment + 1) % radial_segments;
            let current_up = next_section_start + segment;
            let next_up = next_section_start + (segment + 1) % radial_segments;
            
            // First triangle
            indices.push(Triangle::new(current as u32, next as u32, current_up as u32));
            
            // Second triangle
            indices.push(Triangle::new(next as u32, next_up as u32, current_up as u32));
        }
    }
    
    // Add cap for the bottom
    let bottom_center_idx = vertices.len() as u32;
    vertices.push(Point3::new(0.0, 0.0, 0.0));
    
    for segment in 0..radial_segments {
        let current = segment;
        let next = (segment + 1) % radial_segments;
        
        indices.push(Triangle::new(bottom_center_idx, current as u32, next as u32));
    }
    
    // Add cap for the top
    let top_center_idx = vertices.len() as u32;
    vertices.push(Point3::new(0.0, height, 0.0));
    
    let top_start = sections * radial_segments;
    for segment in 0..radial_segments {
        let current = top_start + segment;
        let next = top_start + (segment + 1) % radial_segments;
        
        indices.push(Triangle::new(top_center_idx, next as u32, current as u32));
    }
    
    (vertices, indices)
}

pub fn generate_tree(
    config: BranchConfig,
    seed: Option<u64>,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut generator = TreeGenerator::new(seed);
    
    // Create materials
    let trunk_material = generator.create_trunk_material();



    generator.builder.add_scene(Some("PineTree".to_string()), Some(scene_nodes));
    generator.export(output_path)?;
    
    Ok(())
}
