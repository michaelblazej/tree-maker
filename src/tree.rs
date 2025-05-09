use mesh_tools::{GltfBuilder, material, Triangle, Primitive};
use nalgebra::{Point3, Vector3};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::error::Error;
use std::path::Path;
use std::f32::consts::PI;

use crate::BranchConfig;

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
/// Tuple containing (vertices, indices, normals, uvs) where:
/// - vertices is a Vec<Point3<f32>>
/// - indices is a Vec<Primitive>
/// - normals is a Vec<Vector3<f32>>
/// - uvs is a Vec<[f32; 2]>
pub fn branch_maker(start_radius: f32, end_radius: f32, height: f32, height_segments: u32, radial_segments: u32, noise_level: f32) -> (Vec<Point3<f32>>, Vec<Primitive>, Vec<Vector3<f32>>, Vec<[f32; 2]>) {
    let radial_segments = radial_segments.max(3); // Minimum 3 segments
    let noise_level = noise_level.max(0.0).min(1.0); // Clamp noise level between 0 and 1
    
    let sections = height_segments; // Number of height sections
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    
    // Create a random number generator for noise
    let mut rng = rand::thread_rng();
    
    // Generate vertices
    for section in 0..=sections {
        let section_t = section as f32 / sections as f32;
        let section_radius = start_radius * (1.0 - section_t) + end_radius * section_t;
        let section_z = height * section_t;
        
        for segment in 0..radial_segments {
            let angle = 2.0 * PI * (segment as f32 / radial_segments as f32);
            
            // Add noise to x and z coordinates
            let noise_x = if noise_level > 0.0 { rng.gen_range(-1.0..=1.0) * noise_level * section_radius * 0.3 } else { 0.0 };
            let noise_y = if noise_level > 0.0 { rng.gen_range(-1.0..=1.0) * noise_level * section_radius * 0.3 } else { 0.0 };
            
            // Also add some minor noise to y to make it less uniform
            let noise_z = if noise_level > 0.0 { rng.gen_range(-1.0..=1.0) * noise_level * height * 0.05 } else { 0.0 };
            
            let x = angle.cos() * section_radius + noise_x;
            let y = angle.sin() * section_radius + noise_y;
            let z = section_z + noise_z;
            
            // Calculate normal (pointing outward from the central axis)
            // For a perfect cylinder, normal would just be (cos(angle), sin(angle), 0)
            // But we need to adjust for the noise
            let base_normal_x = angle.cos();
            let base_normal_y = angle.sin();
            
            // Calculate normalized normal vector
            let normal_length = (base_normal_x.powi(2) + base_normal_y.powi(2)).sqrt();
            let normal = Vector3::new(
                base_normal_x / normal_length,
                base_normal_y / normal_length,
                0.0 // For a vertical branch, the Z component of normal is 0
            );
            
            // Calculate texture coordinates (UV)
            let u = segment as f32 / radial_segments as f32;
            let v = section_t;
            
            vertices.push(Point3::new(x, y, z));
            normals.push(normal);
            uvs.push([u, v]);
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
            indices.push(Primitive::Triangle(current as u32, next as u32, current_up as u32));
            
            // Second triangle
            indices.push(Primitive::Triangle(next as u32, next_up as u32, current_up as u32));
        }
    }
    
    // Add cap for the bottom
    let bottom_center_idx = vertices.len() as u32;
    vertices.push(Point3::new(0.0, 0.0, 0.0));
    
    for segment in 0..radial_segments {
        let current = segment;
        let next = (segment + 1) % radial_segments;
        
        indices.push(Primitive::Triangle::new(bottom_center_idx, current as u32, next as u32));
    }
    
    // Add cap for the top
    let top_center_idx = vertices.len() as u32;
    vertices.push(Point3::new(0.0, height, 0.0));
    
    let top_start = sections * radial_segments;
    for segment in 0..radial_segments {
        let current = top_start + segment;
        let next = top_start + (segment + 1) % radial_segments;
        
        indices.push(Primitive::Triangle(top_center_idx, next as u32, current as u32));
    }
    
    (vertices, indices, normals, uvs)
}

pub fn generate_tree(
    config: BranchConfig,
    seed: Option<u64>,
    output_path: Option<&Path>,
) -> Result<(), Box<dyn Error>> {
    let mut generator = TreeGenerator::new(seed);
    
    // Create materials
    let trunk_material = generator.create_trunk_material();
    let leaves_material = generator.create_leaf_material([0.1, 0.6, 0.1, 1.0]); // Green
    
    // Start recursive branch generation from the trunk
    generate_branch_hierarchy(
        &mut generator, 
        &config, 
        None, // No parent for the trunk
        Point3::new(0.0, 0.0, 0.0), // Root position
        trunk_material,
        leaves_material,
        0 // Level 0 = trunk
    );
    
    
    // Use the provided output path or default to "tree.glb"
    let output = match output_path {
        Some(path) => path.to_path_buf(),
        None => std::path::PathBuf::from("tree.glb"),
    };
    
    generator.export(&output)?;
    println!("Tree generated and saved to: {}", output.display());
    
    Ok(())
}

/// Recursively generate branch hierarchy based on the BranchConfig
fn generate_branch_hierarchy(
    /// tree generator
    generator: &mut TreeGenerator,
    /// current branch config
    config: &BranchConfig,
    /// parent node of the current branch
    parent_node: Option<usize>,
    /// position of the current branch
    /// TODO: instead of position, provide a list of points defining the center of the parent branch
    position: Point3<f32>,
    /// material for the trunk and leaves
    trunk_material: usize,
    leaves_material: usize,
    /// level of the current branch
    level: u32,
)  {
    // Generate branch mesh using branch_maker
    let (vertices, triangles, normals, uvs) = branch_maker(
        config.radius, 
        config.radius * config.taper, 
        config.length, 
        config.segments, // Height segments 
        12,  // Radial segments
        config.gnarliness
    );
    
    // Add mesh to the builder
    let mesh_id = generator.builder.create_simple_mesh(
        Some(format!("Mesh_{}", level)), 
        &vertices,
        &triangles,
        &normals,
        &uvs,
        Some(trunk_material),
    );
    
    // Create node for this branch
    let node_name = match level {
        0 => "Trunk".to_string(),
        _ => format!("Branch_L{}_{}", level, rand::random::<u32>() % 100000),
    };
    
    // Add current branch node to scene
    let branch_node = generator.builder.add_node(
        Some(node_name),
        Some(mesh_id),
        Some(position.into()),
        None, // No rotation yet
        None  // No scaling
    );
    
    // Connect to parent if this isn't the trunk
    if let Some(parent_id) = parent_node {
        generator.builder.add_child_to_node(parent_id, branch_node);
    }
    
    // Generate child branches if any
    if config.children > 0 {
        if let Some(child_config) = &config.children_config {
            let child_branch_config = (**child_config).clone();
            
            // Create each child branch based on the number specified
            for i in 0..config.children {
                // Calculate angle for this branch (distribute around parent)
                let angle_radians = 2.0 * std::f32::consts::PI * (i as f32 / config.children as f32);
                
                // Calculate position relative to parent branch end
                let parent_end = Point3::new(
                    position.x,
                    position.y + config.length,
                    position.z
                );
                
                // Apply the branch angle to create an offset direction
                let branch_angle_rad = config.angle * std::f32::consts::PI / 180.0;
                
                // Calculate child branch start position (rotate around parent)
                let child_pos = Point3::new(
                    parent_end.x + angle_radians.cos() * branch_angle_rad.sin() * child_branch_config.radius,
                    parent_end.y,
                    parent_end.z + angle_radians.sin() * branch_angle_rad.sin() * child_branch_config.radius
                );
                
                // Recursively create this child branch and its descendants
                let (child_node, grandchild_nodes) = generate_branch_hierarchy(
                    generator,
                    &child_branch_config,
                    Some(branch_node),
                    child_pos,
                    trunk_material,
                    leaves_material,
                    level + 1
                );
                
            }
        }
    }
    
}
