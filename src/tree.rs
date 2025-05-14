use mesh_tools::{GltfBuilder, Triangle};
use nalgebra::{Point3, Vector3, Vector2, Quaternion, UnitQuaternion, Unit, UnitVector3, Matrix3, Rotation3};
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
        // Handle the case where min == max to avoid the 'cannot sample empty range' error
        if (max - min).abs() < f32::EPSILON {
            return min;
        }
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
            let noise_x = if noise_level > 0.001 { rng.gen_range(-1.0..1.0) * noise_level * section_radius * 0.3 } else { 0.0 };
            let noise_y = if noise_level > 0.001 { rng.gen_range(-1.0..1.0) * noise_level * section_radius * 0.3 } else { 0.0 };
            
            // Less noise in Z direction to avoid significant length changes
            let noise_z = if noise_level > 0.001 { rng.gen_range(-1.0..1.0) * noise_level * height * 0.05 } else { 0.0 };
            
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
    
    // Generate a series of transforms for a more natural branch shape
    let branch_transforms = generate_branch_transforms(
        config.length_segments as usize,    // Number of segments
        config.length / config.length_segments as f32,  // Segment length
        config.gnarliness * 0.2,     // Curvature strength
        config.twist,         // Curvature variation
        Some(generator.rng.gen())    // Random seed
    );
    
    println!("  Generated {} transforms for branch", branch_transforms.len());
    
    // Generate the mesh data for this branch using the transforms
    let (vertices, indices, normals, uvs) = create_transform_based_mesh(
        &branch_transforms,
        config.start_radius,        // Start radius
        config.end_radius,          // End radius
        config.radial_segments as usize, // Radial segments
        config.gnarliness            // Noise level
    );
    
    // Convert UVs from [f32; 2] to Vector2<f32>
    let uvs_vector: Vec<Vector2<f32>> = uvs.iter().map(|uv| Vector2::new(uv[0], uv[1])).collect();
    
    // Create custom mesh for the branch
    let mesh_id = generator.builder.create_custom_mesh(
        Some(format!("Branch_L{}", level)),
        &vertices,
        &indices,
        Some(normals),
        Some(vec![uvs_vector]),     // UVs in the format expected by the API
        Some(trunk_material)        // Material
    );
    
    // Create node for this branch
    let node_name = match level {
        0 => "Trunk".to_string(),
        _ => format!("Branch_L{}_{}", level, rand::random::<u32>() % 100000),
    };
    
    // Generate random rotation angles between min_rotation and max_rotation from config with random sign
    // Ensure min_rot and max_rot are at least 0.1 apart to avoid empty range errors
    let min_rot = config.min_rotation;
    let max_rot = if (config.max_rotation - config.min_rotation) < 0.1 {
        config.min_rotation + 0.1
    } else {
        config.max_rotation
    };
    
    // Generate random rotation with guaranteed non-empty ranges
    let rot_x_deg = generator.rng.gen_range(min_rot..=max_rot) * if generator.rng.gen::<bool>() { 1.0 } else { -1.0 };
    let rot_y_deg = generator.rng.gen_range(min_rot..=max_rot) * if generator.rng.gen::<bool>() { 1.0 } else { -1.0 };
    let rot_z_deg = generator.rng.gen_range(min_rot..=max_rot) * if generator.rng.gen::<bool>() { 1.0 } else { -1.0 };
    
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
    let center_position = position + Vector3::new(0.0,0.0,0.0);
    
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
            println!("  Level {} found child config with start_radius {}", level, (**child_config).start_radius);
            let child_branch_config = (**child_config).clone();
            
            // Create each child branch based on the number specified
            for i in 0..config.children {
                
                // Select a random position along the parent branch for the child
                // Skip the first transform (base) and avoid the very tip for stability
                let valid_transforms = if branch_transforms.len() > 2 {
                    &branch_transforms[1..branch_transforms.len()-1]
                } else {
                    &branch_transforms[..]
                };
                
                let random_index = generator.rng.gen_range(0..valid_transforms.len());
                let random_transform = &valid_transforms[random_index];
                
                // Extract the position from the randomly selected transform
                let child_pos = Point3::new(
                    random_transform.position[0],
                    random_transform.position[1],
                    random_transform.position[2]
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
    pub position: [f32; 3],
    pub rotation: [f32; 4],
}


/// Create a mesh (vertices, indices, normals, uvs) from a series of transforms
/// 
/// # Arguments
/// 
/// * `transforms` - List of transforms defining the path of the branch
/// * `start_radius` - Radius at the base of the branch
/// * `end_radius` - Radius at the tip of the branch
/// * `radial_segments` - Number of segments around the branch circumference
/// * `noise_level` - Amount of random variation (0.0-1.0) to apply to the vertices
/// 
/// # Returns
/// 
/// Tuple containing (vertices, indices, normals, uvs) for the mesh
pub fn create_transform_based_mesh(
    transforms: &[BranchTransform],
    start_radius: f32,
    end_radius: f32,
    radial_segments: usize,
    noise_level: f32
) -> (Vec<Point3<f32>>, Vec<Triangle>, Vec<Vector3<f32>>, Vec<[f32; 2]>) {
    let radial_segments = radial_segments.max(3); // Minimum 3 segments
    let noise_level = noise_level.max(0.0).min(1.0); // Clamp noise level between 0 and 1
    
    let segment_count = transforms.len();
    if segment_count < 2 {
        // Not enough transforms to create a valid mesh
        return (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    }
    
    // Convert transform arrays back to nalgebra types for easier math operations
    let transforms: Vec<(Point3<f32>, UnitQuaternion<f32>)> = transforms.iter().map(|t| {
        let pos = Point3::new(t.position[0], t.position[1], t.position[2]);
        let quat = Quaternion::new(t.rotation[3], t.rotation[0], t.rotation[1], t.rotation[2]);
        let rot = UnitQuaternion::from_quaternion(quat);
        (pos, rot)
    }).collect();
    
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    
    // Create a random number generator for noise
    let mut rng = rand::thread_rng();
    
    // For each transform, create a ring of vertices
    for (i, transform) in transforms.iter().enumerate() {
        let t = i as f32 / (segment_count - 1) as f32; // Parametric value (0 to 1)
        let radius = start_radius * (1.0 - t) + end_radius * t; // Interpolate radius
        
        // Get the position and rotation
        let current_position = transforms[i].0;
        let current_quat = transforms[i].1;
        
        // Create vertices for this ring
        for j in 0..radial_segments {
            let angle = 2.0 * PI * (j as f32 / radial_segments as f32);
            
            // Create a base offset vector around the unit circle
            let base_offset = Vector3::new(angle.cos(), angle.sin(), 0.0);
            
            // Apply noise to the radius
            let noisy_radius = if noise_level > 0.001 {
                // Ensure we have a valid range to sample from
                radius * (1.0 + rng.gen_range(-noise_level..noise_level) * 0.3)
            } else {
                radius
            };
            
            // Scale and rotate the offset vector
            let offset = current_quat * base_offset.scale(noisy_radius);
            
            // Final vertex position
            let vertex = current_position + offset;
            
            // Calculate normal (pointing outward from center)
            let normal = offset.normalize();
            
            // Calculate UV coordinates
            let u = j as f32 / radial_segments as f32;
            let v = t;
            
            vertices.push(vertex);
            normals.push(normal);
            uvs.push([u, v]);
        }
    }
    
    // Create triangles between rings
    for i in 0..(segment_count - 1) {
        let ring_start = i * radial_segments;
        let next_ring_start = (i + 1) * radial_segments;
        
        for j in 0..radial_segments {
            let current = ring_start + j;
            let next = ring_start + ((j + 1) % radial_segments);
            let current_up = next_ring_start + j;
            let next_up = next_ring_start + ((j + 1) % radial_segments);
            
            // First triangle
            indices.push(Triangle::new(current as u32, next as u32, current_up as u32));
            
            // Second triangle
            indices.push(Triangle::new(next as u32, next_up as u32, current_up as u32));
        }
    }
    
    // Add cap for the bottom
    let bottom_center_idx = vertices.len() as u32;
    vertices.push(transforms[0].0);
    normals.push(Vector3::new(0.0, 0.0, -1.0));
    uvs.push([0.5, 0.0]);
    
    for j in 0..radial_segments {
        let current = j;
        let next = (j + 1) % radial_segments;
        
        indices.push(Triangle::new(bottom_center_idx, current as u32, next as u32));
    }
    
    // Handle top of the branch based on end radius
    if end_radius < 0.0001 {
        // Create a single vertex at the tip
        let tip_idx = vertices.len() as u32;
        vertices.push(transforms[segment_count - 1].0);
        
        // Calculate the direction from the second-to-last point to the last point
        let direction = if segment_count > 2 {
            (transforms[segment_count - 1].0 - transforms[segment_count - 2].0).normalize()
        } else {
            Vector3::new(0.0, 0.0, 1.0)
        };
        
        // Use this direction as the normal for the tip
        normals.push(direction);
        uvs.push([0.5, 1.0]);
        
        // Connect the last ring to the tip point
        let top_start = (segment_count - 1) * radial_segments;
        for j in 0..radial_segments {
            let current = top_start + j;
            let next = top_start + ((j + 1) % radial_segments);
            
            indices.push(Triangle::new(tip_idx, next as u32, current as u32));
        }
    } else {
        // Normal cap for non-zero end radius
        let top_center_idx = vertices.len() as u32;
        vertices.push(transforms[segment_count - 1].0);
        normals.push(Vector3::new(0.0, 0.0, 1.0));
        uvs.push([0.5, 1.0]);
        
        let top_start = (segment_count - 1) * radial_segments;
        for j in 0..radial_segments {
            let current = top_start + j;
            let next = top_start + ((j + 1) % radial_segments);
            
            indices.push(Triangle::new(top_center_idx, next as u32, current as u32));
        }
    }
    
    // Return the generated mesh data
    (vertices, indices, normals, uvs)
}

/// Generate a list of transforms along a branch with natural growth
/// 
/// # Arguments
/// 
/// * `segment_count` - Number of segments in the branch
/// * `segment_length` - Length of each segment
/// * `curvature_strength` - Strength of the branch curvature
/// * `curvature_variation` - Variation in curvature
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
    // Initialize the random number generator with seed if provided
    let mut rng = if let Some(s) = seed {
        ChaCha8Rng::seed_from_u64(s)
    } else {
        ChaCha8Rng::from_entropy()
    };
    
    let mut transforms = Vec::with_capacity(segment_count);
    
    // Set up the initial position and direction
    let mut position = Vector3::new(0.0, 0.0, 0.0);
    let initial_direction = Vector3::new(0.0, 0.0, 1.0);
    let mut cumulative_rotation = UnitQuaternion::identity();
    
    // Add the initial transform
    transforms.push(BranchTransform {
        position: [position.x, position.y, position.z],
        rotation: [0.0, 0.0, 0.0, 1.0], // Identity quaternion (w=1)
    });
    
    // Generate the transforms for the remaining segments
    for _ in 1..segment_count {
        // Create a small random rotation to apply to the growth direction
        // Use a small value for curvature (0.01) if provided value is too small or zero
        let actual_curvature = if curvature_strength < 0.001 { 0.01 } else { curvature_strength };
        
        // Use a minimum value for variation to avoid empty ranges
        let actual_variation = if curvature_variation < 0.001 { 0.1 } else { curvature_variation };
        
        // Generate rotation angles for each axis (pitch, yaw, roll)
        // Use hardcoded ranges to avoid empty range errors
        let pitch = actual_curvature * (rng.gen::<f32>() * 2.0 - 1.0) * actual_variation;
        let yaw = actual_curvature * (rng.gen::<f32>() * 2.0 - 1.0) * actual_variation;
        let roll = actual_curvature * (rng.gen::<f32>() * 2.0 - 1.0) * actual_variation * 0.5;
        
        // Create rotation quaternion from the random angles
        let segment_rotation = UnitQuaternion::from_euler_angles(pitch, yaw, roll);
        
        // Apply the rotation to our cumulative rotation
        cumulative_rotation = cumulative_rotation * segment_rotation;
        
        // Calculate new position by moving in the direction determined by the cumulative rotation
        let direction = cumulative_rotation * initial_direction;
        position += direction * segment_length;
        
        // Extract the quaternion components
        let quat = cumulative_rotation.into_inner();
        
        // Add the transform for this segment
        transforms.push(BranchTransform {
            position: [position.x, position.y, position.z],
            rotation: [quat.i, quat.j, quat.k, quat.w],
        });
    }
    
    transforms
}

// L-system approach no longer used - replaced with continuous growth vector
