pub mod binary_node;
pub mod noise;
use std::{collections::HashMap, u32};

use binary_node::*;

use bevy::{
    app::DynEq,
    log::warn,
    math::Vec3,
    pbr::{Material, StandardMaterial},
    prelude::{Mesh, Triangle2d, Triangle3d},
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::BindGroupLayoutEntry,
    },
};
use bevy_rapier3d::prelude::shape_views::TriangleView;
use bevy_tnua::math::Vector2;

#[derive(Debug)]
pub struct TerrainMeshData {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
}

pub trait PlaneSampler {
    fn get(&self, x: f32, y: f32) -> f32;
}

impl TerrainMeshData {
    pub fn into_mesh(&self, enable_wireframe: bool, size: f32) -> Mesh {
        let topology = if enable_wireframe {
            PrimitiveTopology::LineList
        } else {
            PrimitiveTopology::TriangleList
        };

        let indices_len = if enable_wireframe {
            self.indices.len() * 2
        } else {
            self.indices.len()
        };

        let mut mesh = Mesh::new(topology, RenderAssetUsages::default());

        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut uvs = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut colors: Vec<[f32; 3]> = Vec::new();

        vertices.reserve(self.vertices.len());
        colors.reserve(vertices.len());
        indices.reserve(indices_len);
        // let grad = palette::Mic Mix::new(vec![
        //     Hsv::from(),
        //     Hsv::from(),
        // ]);

        for vertex in &self.vertices {
            vertices.push([vertex.x, vertex.y, vertex.z]);
            uvs.push([vertex.x / size, vertex.z / size]);
        }

        let triangle_number = self.indices.len() / 3;

        if enable_wireframe {
            for i in 0..triangle_number {
                for j in &[0, 1, 1, 2, 2, 0] {
                    indices.push(self.indices[i * 3 + j]);
                }
            }
        } else {
            for i in 0..triangle_number {
                for j in 0..3 {
                    indices.push(self.indices[i * 3 + j]);
                }
            }
        }

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        // mesh.insert_attribute(Material, values)
        // mesh.insert_attribute(StandardMaterial::A, colors);
        mesh.insert_indices(Indices::U32(indices));

        if !enable_wireframe {
            mesh.compute_normals();
        }

        mesh
    }
}

pub fn build_terrain_from_sampler(
    sampler: &impl PlaneSampler,
    height_multiplier: f32,
    size: f32,
    error_threshold: f32,
) -> TerrainMeshData {
    let grid_size = size + 1.;
    let errors = get_errors_vec(sampler, grid_size);
    // warn!("got errors vec: \n{errors:?}");
    let avg = errors.iter().fold(0., |acc, e| acc + e) / errors.len() as f32;
    warn!("avg error: {}", avg);

    let mut vertices = Vec::<Vec3>::new();
    let mut indices = Vec::<u32>::new();
    let mut vertices_array_position = HashMap::<u32, usize>::new();

    let nodes = select_nodes(size, &errors, error_threshold);
    // debug!("building terrain from nodes: {nodes:?}");

    for node in nodes {
        let triangle_coords = node.triangle_coords(size);
        let new_vertices = &[
            &triangle_coords.vertices[0],
            &triangle_coords.vertices[1],
            &triangle_coords.vertices[2],
        ];

        for new_vertex in new_vertices {
            let vertex_id = (new_vertex[1] * size + new_vertex[0]) as u32;

            let vertex_index = match vertices_array_position.get(&vertex_id) {
                Some(i) => i.to_owned(),
                None => {
                    let new_vertex_index = vertices.len();
                    vertices_array_position.insert(vertex_id, new_vertex_index);

                    let vertex_height =
                        sample_corner_mean(sampler, &size, **new_vertex) * height_multiplier;

                    let new_vertex_3d =
                        Vec3::new(new_vertex[0] as f32, vertex_height, new_vertex[1] as f32);
                    vertices.push(new_vertex_3d);
                    new_vertex_index.to_owned()
                }
            };
            indices.push(vertex_index as u32);
        }
    }

    TerrainMeshData { vertices, indices }
}

const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}
fn log_2(x: u32) -> u32 {
    num_bits::<u32>() as u32 - x.leading_zeros() - 1
}

fn get_errors_vec(sampler: &impl PlaneSampler, grid_size: f32) -> Vec<f32> {
    // let grid_grid_size = size + 1.;
    let number_of_triangles = grid_size as u32 * grid_size as u32 * 2 - 2;
    let number_of_levels = log_2(grid_size as u32) * 2;
    let last_level = number_of_levels - 1;

    let last_level_index_start = BinaryNode::level_start_index(last_level);

    let mut errors_vec = Vec::new();
    errors_vec.resize((grid_size * grid_size) as usize, 0.0f32);

    for idx in (0..number_of_triangles).rev() {
        let node = BinaryNode::from_triangle_index(idx);

        let midpoint = node.midpoint_pixel_coords(grid_size);

        let triangle_coords = node.triangle_coords(grid_size);
        let h0 = sample_corner_mean(sampler, &grid_size, triangle_coords.vertices[0]);
        let h1 = sample_corner_mean(sampler, &grid_size, triangle_coords.vertices[1]);
        let midpoint_interpolated = (h1 + h0) / 2.0;
        let midpoint_height = sample_corner_mean(sampler, &grid_size, midpoint);

        let this_triangle_error = (midpoint_interpolated - midpoint_height).abs();

        let this_triangle_mid_point_error_vec_index = node.errors_vec_index(grid_size);

        if idx >= last_level_index_start {
            errors_vec[this_triangle_mid_point_error_vec_index] = this_triangle_error;
        } else {
            let (left_child, right_child) = node.children_ids();

            let right_errors_vec_index = right_child.errors_vec_index(grid_size);
            let left_errors_vec_index = left_child.errors_vec_index(grid_size);

            let prev_error = errors_vec[this_triangle_mid_point_error_vec_index];
            let right_error = errors_vec[right_errors_vec_index];
            let left_error = errors_vec[left_errors_vec_index];

            errors_vec[this_triangle_mid_point_error_vec_index] = prev_error
                .max(left_error)
                .max(right_error)
                .max(this_triangle_error);
        }
    }

    errors_vec
}

pub fn sample_corner_mean(sampler: &impl PlaneSampler, size: &f32, corner_u32: Vector2) -> f32 {
    let mut new_corner = corner_u32;

    if new_corner[0] >= *size {
        new_corner[0] = size - 1.;
    }

    if new_corner[1] >= *size {
        new_corner[1] = size - 1.;
    }

    sampler.get(new_corner[0], new_corner[1])
}

fn select_nodes(size: f32, errors_vec: &Vec<f32>, error_threshold: f32) -> Vec<BinaryNode> {
    let mut nodes = Vec::<BinaryNode>::new();

    select_nodes_for_heightmap(size, errors_vec, &mut nodes, 0, error_threshold);
    select_nodes_for_heightmap(size, errors_vec, &mut nodes, 1, error_threshold);
    nodes
}

fn select_nodes_for_heightmap(
    size: f32,
    errors_vec: &Vec<f32>,
    nodes: &mut Vec<BinaryNode>,
    triangle_index: u32,
    error_threshold: f32,
) {
    let grid_size = size + 1.;
    let side = size;

    let node = BinaryNode::from_triangle_index(triangle_index);

    let (left_child, right_child) = node.children_ids();

    warn!(
        "parent: {:?}\nleft: {:?}\nright: {:?}",
        node, left_child, right_child
    );

    let number_of_last_level_triangles = side * side * 2.;
    let number_of_triangles = side * side * 2. - 2. + number_of_last_level_triangles;
    warn!("number of triangles: {}", number_of_triangles);

    let has_children = right_child.triangle_index() < number_of_triangles as u32;
    let leaf_node = !has_children;

    let this_triangle_errors_vec_index = node.errors_vec_index(grid_size);
    warn!("this triangle error index: {this_triangle_errors_vec_index}");
    let this_triangle_error = errors_vec[this_triangle_errors_vec_index];

    warn!("this triangle error: {this_triangle_error}");

    let error_within_threshold = this_triangle_error <= error_threshold;

    if error_within_threshold || leaf_node {
        warn!("adding to nodes: {}", node.as_ref());
        nodes.push(node);
    } else {
        warn!(
            "node: {} error is too high, iterating through children triangles\nleft: {}\nright: {}\n",
            node.as_ref(), left_child.triangle_index(), right_child.triangle_index()
        );
        select_nodes_for_heightmap(
            size,
            errors_vec,
            nodes,
            left_child.triangle_index(),
            error_threshold,
        );
        select_nodes_for_heightmap(
            size,
            errors_vec,
            nodes,
            right_child.triangle_index(),
            error_threshold,
        );
    }
}

mod tests {
    use bevy::scene::ron::error;
    use noise::{Fbm, Perlin};

    use crate::rtin::{noise::NoiseSampler, BinaryNode, PlaneSampler};

    use super::{get_errors_vec, select_nodes};

    #[test]
    fn errors_vec_correct() {
        let noise = Fbm::<Perlin>::new(69);
        let size = 4.;
        let sampler = NoiseSampler::single_layer(noise);
        let errors = get_errors_vec(&sampler, size);
        assert_eq!(16, errors.len());
        println!("errors: {:?}", errors);
    }

    #[test]
    fn select_nodes_works() {
        let size = 4.;

        let grid_size = size as usize + 1;

        let mut errors_vec = vec![0.; grid_size * grid_size];
        errors_vec[13] = 0.5;
        errors_vec[12] = 0.5;
        let error_threshold = 0.4;
        let nodes = select_nodes(size, &mut errors_vec, error_threshold);
        let expected_nodes: Vec<BinaryNode> = vec![4.into(), 5.into(), 6.into(), 7.into()];
        assert_eq!(expected_nodes, nodes);

        let mut errors_vec = vec![0.; grid_size * grid_size];
        errors_vec[12] = 0.5;
        errors_vec[14] = 0.5;
        let error_threshold = 0.4;
        let nodes = select_nodes(size, &mut errors_vec, error_threshold);
        let expected_nodes: Vec<BinaryNode> =
            vec![4.into(), 5.into(), 6.into(), 14.into(), 15.into()];

        println!("{:?}", nodes);
        assert_eq!(expected_nodes, nodes);
    }
}
