use mesh_tools::{GltfBuilder, Triangle};
use nalgebra::{Point3, Vector3, Quaternion, UnitQuaternion, Unit, UnitVector3, Matrix3, Rotation3};
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
/// - indices is a Vec<Triangle>
/// - normals is a Vec<Vector3<f32>>
/// - uvs is a Vec<[f32; 2]>
pub fn branch_maker(start_radius: f32, end_radius: f32, height: f32, height_segments: u32, radial_segments: u32, noise_level: f32) -> (Vec<Point3<f32>>, Vec<Triangle>, Vec<Vector3<f32>>, Vec<[f32; 2]>) {
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
    
    // Create a root node for the tree
    let root_node = generator.builder.add_node(
        Some("Tree".to_string()),
        None,
        None,
        None,
        None
    );
    
    // Start recursive branch generation from the trunk
    generate_branch_hierarchy(
        &mut generator, 
        &config, 
        Some(root_node), // Root node as parent
        Point3::new(0.0, 0.0, 0.0), // Root position
        trunk_material,
        leaves_material,
        0 // Level 0 = trunk
    );
    
    // Create a scene with the root node
    generator.builder.add_scene(Some("Tree".to_string()), Some(vec![root_node]));
    
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
    generator: &mut TreeGenerator,
    config: &BranchConfig,
    parent_node: Option<usize>,
    position: Point3<f32>,
    trunk_material: usize,
    leaves_material: usize,
    level: u32,
)  {
    println!("Generating branch at level {}, with {} children", level, config.children);
    println!("  children_config is {}", if config.children_config.is_some() { "Some" } else { "None" });
    // Generate branch mesh using branch_maker
    let (vertices, indices, normals, uvs) = branch_maker(
        config.radius, 
        config.radius * config.taper, 
        config.length, 
        config.segments, // Height segments 
        12,  // Radial segments
        config.gnarliness
    );
    
    // Create cylinder mesh using the vertices and triangles directly
    let mesh_id = generator.builder.create_cylinder(
        config.radius * config.taper, // Top radius
        config.radius,               // Bottom radius
        config.length,               // Height
        12,                          // Radial segments
        config.segments as usize,    // Height segments
        false,                       // Not open-ended
        Some(trunk_material)         // Material
    );
    
    // Create node for this branch
    let node_name = match level {
        0 => "Trunk".to_string(),
        _ => format!("Branch_L{}_{}", level, rand::random::<u32>() % 100000),
    };
    
    // Generate random rotation angles between 30 and 90 degrees
    let rot_x_deg = generator.random_f32(30.0, 90.0);
    let rot_y_deg = generator.random_f32(30.0, 90.0);
    let rot_z_deg = generator.random_f32(30.0, 90.0);
    
    // Convert to radians
    let rot_x = rot_x_deg * std::f32::consts::PI / 180.0;
    let rot_y = rot_y_deg * std::f32::consts::PI / 180.0;
    let rot_z = rot_z_deg * std::f32::consts::PI / 180.0;
    
    // Create rotation quaternion
    let rotation = UnitQuaternion::from_euler_angles(rot_x, rot_y, rot_z);
    
    // Extract quaternion components in the order expected by GLTF (x, y, z, w)
    let quat = rotation.into_inner();
    let gltf_rotation = [quat.i, quat.j, quat.k, quat.w];

    // Add current branch node to scene
    let center_position = position + Vector3::new(0.0,config.length/2.0,0.0);
    
    let branch_node = generator.builder.add_node(
        Some(node_name),
        Some(mesh_id),
        Some(center_position.into()),
        Some(gltf_rotation),
        None  // No scaling
    );
    
    // Connect to parent if this isn't the trunk
    if let Some(parent_id) = parent_node {
        generator.builder.add_child_to_node(parent_id, branch_node);
    }
    
    // Generate child branches if any
    if config.children > 0 {
        println!("  Level {} has {} children to generate", level, config.children);
        if let Some(child_config) = &config.children_config {
            println!("  Level {} found child config with radius {}", level, (**child_config).radius);
            let child_branch_config = (**child_config).clone();
            
            // Create each child branch based on the number specified
            for i in 0..config.children {
                // Calculate angle for this branch (distribute around parent)
                let angle_radians = 2.0 * std::f32::consts::PI * (i as f32 / config.children as f32);
                
                // Calculate position relative to parent branch end
                let parent_end = Point3::new(
                    position.x,
                    position.y + config.length/2.0,
                    position.z
                );
                
                // Apply the branch angle to create an offset direction
                let branch_angle_rad = config.angle * std::f32::consts::PI / 180.0;
                
                // Calculate child branch start position (rotate around parent)
                let child_pos = Point3::new(
                    0.0,
                    generator.random_f32(0.0, parent_end.y),
                    0.0
                );
                println!("  Child position: ({}, {}, {})", child_pos.x, child_pos.y, child_pos.z);
                
                // Recursively create this child branch and its descendants
                println!("  Creating child {} of {} for level {}", i+1, config.children, level);
                generate_branch_hierarchy(
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

/// A transform representing position and rotation in 3D space
#[derive(Debug, Clone)]
pub struct BranchTransform {
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
}

/// Generate a list of transforms along a curving branch
/// 
/// # Arguments
/// 
/// * `segment_count` - Number of segments in the branch
/// * `segment_length` - Length of each segment
/// * `curvature_strength` - How much the branch can curve (higher values = more curvy)
/// * `curvature_variation` - How much rotation around the branch axis can occur
/// * `seed` - Optional random seed for reproducibility
/// 
/// # Returns
/// 
/// A vector of BranchTransform containing position and rotation for each segment
pub fn generate_branch_transforms(
    segment_count: usize,
    segment_length: f32,
    curvature_strength: f32,
    curvature_variation: f32,
    seed: Option<u64>
) -> Vec<BranchTransform> {
    // Create an RNG with the given seed or use a random one
    let mut rng = match seed {
        Some(s) => ChaCha8Rng::seed_from_u64(s),
        None => ChaCha8Rng::from_entropy(),
    };
    
    let mut transforms = Vec::with_capacity(segment_count);
    
    // Initial position and direction
    let mut position = Point3::new(0.0, 0.0, 0.0);
    let mut direction = Vector3::new(0.0, 0.0, 1.0);
    let up = Vector3::new(0.0, 1.0, 0.0);
    
    for _ in 0..segment_count {
        // Compute small random curvature (pitch, yaw, roll)
        let pitch = rng.gen_range(-curvature_strength..=curvature_strength);
        let yaw = rng.gen_range(-curvature_strength..=curvature_strength);
        let roll = rng.gen_range(-curvature_variation..=curvature_variation);
        
        // Apply curvature to the direction using Euler angles
        // First, roll around Z
        let roll_rot = Rotation3::from_euler_angles(0.0, 0.0, roll);
        // Then pitch around X
        let pitch_rot = Rotation3::from_euler_angles(pitch, 0.0, 0.0);
        // Finally, yaw around Y
        let yaw_rot = Rotation3::from_euler_angles(0.0, yaw, 0.0);
        
        // Combine rotations and apply to direction
        let combined_rot = yaw_rot * pitch_rot * roll_rot;
        direction = combined_rot * direction;
        
        // Normalize the direction vector
        let direction_norm = direction.magnitude();
        if direction_norm > 1e-6 {
            direction /= direction_norm;
        }
        
        // Create rotation that aligns Z+ with current direction
        // First, calculate X axis by crossing up with Z
        let mut x_axis = up.cross(&direction);
        
        // If x_axis is too small (happens when direction is parallel to up),
        // use a default X axis
        if x_axis.magnitude() < 1e-6 {
            x_axis = Vector3::new(1.0, 0.0, 0.0);
        } else {
            x_axis = x_axis.normalize();
        }
        
        // Calculate Y axis by crossing Z with X
        let y_axis = direction.cross(&x_axis).normalize();
        
        // Create rotation matrix from orthogonal axes
        let rot_matrix = Matrix3::from_columns(&[x_axis, y_axis, direction]);
        
        // Convert to quaternion
        let rot_quat = UnitQuaternion::from_matrix(&rot_matrix);
        
        // Add transform to the list
        transforms.push(BranchTransform {
            position,
            rotation: rot_quat,
        });
        
        // Move to next position
        position += direction.scale(segment_length);
    }
    
    transforms
}
