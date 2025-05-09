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
    fn branch_maker(&mut self, start_radius: f32, end_radius: f32, height: f32, height_segments: u32, radial_segments: u32, noise_level: f32) -> (Vec<Point3<f32>>, Vec<Triangle>) {
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
}

/// Creates a custom branch mesh with specified parameters
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
/// Tuple containing (vertices, indices) where vertices is a Vec<[f32; 3]> and indices is a Vec<u32>
pub fn branch_maker(
    start_radius: f32,
    end_radius: f32,
    height: f32,
    height_segments: u32,
    radial_segments: u32,
    noise_level: f32,
    seed: Option<u64>,
) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut generator = TreeGenerator::new(seed);
    let (points, triangles) = generator.branch_maker(start_radius, end_radius, height, height_segments, radial_segments, noise_level);
    
    // Convert Point3<f32> to [f32; 3]
    let vertices = points.into_iter().map(|point| [point.x, point.y, point.z]).collect();
    
    // Convert Triangle to Vec<u32>
    let mut indices = Vec::new();
    for triangle in triangles {
        indices.push(triangle.a);
        indices.push(triangle.b);
        indices.push(triangle.c);
    }
    
    (vertices, indices)
}

// Pine tree generation
pub fn generate_pine_tree(
    height: f32,
    branch_density: f32,
    detail_level: u32,
    seed: Option<u64>,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut generator = TreeGenerator::new(seed);
    
    // Create materials
    let trunk_material = generator.create_trunk_material();
    let needle_material = generator.create_leaf_material([0.0, 0.5, 0.2, 1.0]); // Dark green
    
    // Create trunk
    let trunk_height = height * 0.9;
    let trunk_radius = height * 0.05;
    let trunk_mesh = generator.builder.create_cylinder(
        trunk_radius * 0.7, // Top radius
        trunk_radius,       // Bottom radius
        trunk_height,
        8,                  // Radial segments
        1,                  // Height segments
        false,              // Not open-ended
        Some(trunk_material),
    );
    
    // Add trunk to scene
    let trunk_node = generator.builder.add_node(
        Some("Trunk".to_string()),
        Some(trunk_mesh),
        Some(Point3::new(0.0, trunk_height / 2.0, 0.0).into()),
        None,
        None,
    );
    
    // Create foliage layers
    let num_layers = 5;
    let mut foliage_nodes = Vec::new();
    
    for i in 0..num_layers {
        let layer_radius = trunk_radius * 4.0 * (1.0 - (i as f32 / num_layers as f32) * 0.8);
        let layer_height = 0.3 * height;
        let y_position = height * 0.3 + (height * 0.7) * (i as f32 / num_layers as f32);
        
        let cone_mesh = generator.builder.create_cone(
            layer_radius,
            layer_height,
            16,           // Radial segments
            1,            // Height segments
            false,        // Not open-ended
            Some(needle_material),
        );
        
        let cone_node = generator.builder.add_node(
            Some(format!("PineLayer_{}", i)),
            Some(cone_mesh),
            Some(Point3::new(0.0, y_position, 0.0).into()),
            None,
            None,
        );
        
        foliage_nodes.push(cone_node);
    }
    
    // Create scene with all nodes
    let mut scene_nodes = vec![trunk_node];
    scene_nodes.extend(foliage_nodes);
    
    generator.builder.add_scene(Some("PineTree".to_string()), Some(scene_nodes));
    generator.export(output_path)?;
    
    Ok(())
}

// Oak tree generation
pub fn generate_oak_tree(
    height: f32,
    branch_density: f32,
    detail_level: u32,
    seed: Option<u64>,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut generator = TreeGenerator::new(seed);
    
    // Create materials
    let trunk_material = generator.create_trunk_material();
    let leaf_material = generator.create_leaf_material([0.1, 0.6, 0.1, 1.0]); // Green
    
    // Create trunk
    let trunk_height = height * 0.7;
    let trunk_radius = height * 0.06;
    let trunk_mesh = generator.builder.create_cylinder(
        trunk_radius * 0.8, // Top radius
        trunk_radius,       // Bottom radius
        trunk_height,
        8,                  // Radial segments
        1,                  // Height segments
        false,              // Not open-ended
        Some(trunk_material),
    );
    
    // Add trunk to scene
    let trunk_node = generator.builder.add_node(
        Some("Trunk".to_string()),
        Some(trunk_mesh),
        Some(Point3::new(0.0, trunk_height / 2.0, 0.0).into()),
        None,
        None,
    );
    
    // Create main canopy
    let canopy_radius = height * 0.4;
    let canopy_mesh = generator.builder.create_sphere(
        canopy_radius,
        16,              // Width segments
        12,              // Height segments
        Some(leaf_material),
    );
    
    let canopy_node = generator.builder.add_node(
        Some("Canopy".to_string()),
        Some(canopy_mesh),
        Some(Point3::new(0.0, trunk_height + canopy_radius * 0.7, 0.0).into()),
        None,
        None,
    );
    
    // Create scene with all nodes
    let scene_nodes = vec![trunk_node, canopy_node];
    
    generator.builder.add_scene(Some("OakTree".to_string()), Some(scene_nodes));
    generator.export(output_path)?;
    
    Ok(())
}

// Willow tree generation
pub fn generate_willow_tree(
    height: f32,
    branch_density: f32,
    detail_level: u32,
    seed: Option<u64>,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut generator = TreeGenerator::new(seed);
    
    // Create materials
    let trunk_material = generator.create_trunk_material();
    let leaf_material = generator.create_leaf_material([0.3, 0.7, 0.3, 1.0]); // Light green
    
    // Create trunk
    let trunk_height = height * 0.6;
    let trunk_radius = height * 0.05;
    let trunk_mesh = generator.builder.create_cylinder(
        trunk_radius * 0.6, // Top radius
        trunk_radius,       // Bottom radius
        trunk_height,
        8,                  // Radial segments
        1,                  // Height segments
        false,              // Not open-ended
        Some(trunk_material),
    );
    
    // Add trunk to scene
    let trunk_node = generator.builder.add_node(
        Some("Trunk".to_string()),
        Some(trunk_mesh),
        Some(Point3::new(0.0, trunk_height / 2.0, 0.0).into()),
        None,
        None,
    );
    
    // Create drooping canopy shape
    let canopy_radius = height * 0.5;
    let canopy_height = height * 0.7;
    let canopy_mesh = generator.builder.create_sphere(
        canopy_radius,
        16,              // Width segments
        12,              // Height segments
        Some(leaf_material),
    );
    
    // Position canopy with a scale to make it droopy
    let canopy_node = generator.builder.add_node(
        Some("Canopy".to_string()),
        Some(canopy_mesh),
        Some(Point3::new(0.0, trunk_height + canopy_height * 0.15, 0.0).into()),
        None,
        Some(Vector3::new(1.0, 0.7, 1.0).into()), // Flatten vertically
    );
    
    // Create scene with all nodes
    let scene_nodes = vec![trunk_node, canopy_node];
    
    generator.builder.add_scene(Some("WillowTree".to_string()), Some(scene_nodes));
    generator.export(output_path)?;
    
    Ok(())
}

// Palm tree generation
pub fn generate_palm_tree(
    height: f32,
    branch_density: f32,
    detail_level: u32,
    seed: Option<u64>,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut generator = TreeGenerator::new(seed);
    
    // Create materials
    let trunk_material = generator.create_trunk_material();
    let leaf_material = generator.create_leaf_material([0.1, 0.5, 0.1, 1.0]); // Green
    
    // Create tall, thin trunk
    let trunk_height = height * 0.9;
    let trunk_radius = height * 0.03;
    let trunk_mesh = generator.builder.create_cylinder(
        trunk_radius,     // Top radius
        trunk_radius * 1.2, // Bottom radius
        trunk_height,
        8,                // Radial segments
        1,                // Height segments
        false,            // Not open-ended
        Some(trunk_material),
    );
    
    // Add trunk to scene
    let trunk_node = generator.builder.add_node(
        Some("Trunk".to_string()),
        Some(trunk_mesh),
        Some(Point3::new(0.0, trunk_height / 2.0, 0.0).into()),
        None,
        None,
    );
    
    // Create palm fronds (here we simplify as cones)
    let mut frond_nodes = Vec::new();
    let num_fronds = 7;
    
    for i in 0..num_fronds {
        let angle = (i as f32 / num_fronds as f32) * 2.0 * std::f32::consts::PI;
        let frond_length = height * 0.25;
        
        let frond_mesh = generator.builder.create_cone(
            frond_length * 0.2, // Base radius
            frond_length,
            6,                // Radial segments
            1,                // Height segments
            false,            // Not open-ended
            Some(leaf_material),
        );
        
        // Position and rotate the frond outward from the top of the trunk
        let frond_node = generator.builder.add_node(
            Some(format!("Frond_{}", i)),
            Some(frond_mesh),
            Some(Point3::new(
                angle.cos() * frond_length * 0.2,
                trunk_height + frond_length * 0.5,
                angle.sin() * frond_length * 0.2,
            ).into()),
            // Convert Vector3 to quaternion rotation
            Some([angle.cos() * 0.5, 0.0, angle.sin() * 0.5, 1.0]),
            None,
        );
        
        frond_nodes.push(frond_node);
    }
    
    // Create scene with all nodes
    let mut scene_nodes = vec![trunk_node];
    scene_nodes.extend(frond_nodes);
    
    generator.builder.add_scene(Some("PalmTree".to_string()), Some(scene_nodes));
    generator.export(output_path)?;
    
    Ok(())
}
