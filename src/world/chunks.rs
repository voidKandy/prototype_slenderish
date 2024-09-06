use super::GROUND_Y;
use bevy::{
    color::palettes::css::{BLUE, BROWN, GREEN, YELLOW},
    prelude::*,
    utils::HashMap,
};
use bevy_rapier3d::prelude::*;
use rand::Rng;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkType {
    Lake,
    Field,
    Forest,
}

#[derive(Debug)]
pub struct Chunk {
    pub typ: ChunkType,
    // coordinates of the bottom left corner of the chunk
    pub x: f32,
    pub z: f32,
}

// Simply a grid of chunks mapped over the 3D space
// The hash IS NOT their coordinates in space, but their position on the grid
#[derive(Resource, Debug)]
pub struct ChunkMap(pub HashMap<(usize, usize), Chunk>);

const CHUNK_SIZE: f32 = 50.0;

fn distance_between_vec3(vec: Vec3, other: Vec3) -> f32 {
    let x = vec.x - other.x;
    let x = x * x;
    let z = vec.z - other.z;
    let z = z * z;
    (x + z).sqrt()
}

fn x_and_y_from_pos(pos: &Vec3) -> (usize, usize) {
    let (mut x, mut z) = (
        (pos.x / CHUNK_SIZE).round_ties_even() as usize,
        (pos.z / CHUNK_SIZE).round_ties_even() as usize,
    );

    if pos.x % CHUNK_SIZE != 0. {
        x += 1;
    }

    if pos.z % CHUNK_SIZE != 0. {
        z += 1;
    }

    (x, z)
}

pub fn setup_chunk_map(mut commands: Commands) {
    let map = ChunkMap::generate_map(3);
    commands.insert_resource(map);
}

impl ChunkMap {
    fn generate_map(width: usize) -> Self {
        let mut map = HashMap::new();
        let mut rng = rand::thread_rng();
        let all_types = vec![ChunkType::Forest, ChunkType::Field, ChunkType::Lake];
        for i in 0..width {
            let x = i as f32 * (CHUNK_SIZE - 1.);
            for k in 0..width {
                let z = k as f32 * (CHUNK_SIZE - 1.);
                let typ = all_types[rng.gen_range(0..all_types.len())].clone();
                let chunk = Chunk { typ, x, z };
                map.insert((i, k), chunk);
            }
        }
        Self(map)
    }
    pub fn get_chunk(&self, pos: &Vec3) -> &Chunk {
        let coords = x_and_y_from_pos(pos);
        self.0.get(&coords).unwrap()
    }
}

struct MeshResult {
    ow: Vec<(Mesh, Collider)>,
    uw: Vec<(Mesh, Collider)>,
}

impl Chunk {
    pub fn x_range(&self) -> Range<i32> {
        self.x as i32..self.x as i32 + CHUNK_SIZE as i32
    }
    pub fn z_range(&self) -> Range<i32> {
        self.z as i32..self.z as i32 + CHUNK_SIZE as i32
    }
    pub fn spawn_trees(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        let mut rng = rand::thread_rng();
        let amt_of_trees = 25;
        let mut all_tree_transforms: Vec<Transform> = vec![];
        for _ in 0..=amt_of_trees {
            // let (x, y, z) = (
            //     trans.translation.x,
            //     trans.translation.y,
            //     trans.translation.z,
            // );

            // add some margins
            let min_x = self.x_range().start as f32 + MIN_GAP_BETWEEN_TREES;
            let min_z = self.z_range().start as f32 + MIN_GAP_BETWEEN_TREES;
            let max_x = self.x_range().end as f32 - MIN_GAP_BETWEEN_TREES;
            let max_z = self.z_range().end as f32 - MIN_GAP_BETWEEN_TREES;

            let mut pos_x = rng.gen_range(min_x..=max_x);
            let mut pos_z = rng.gen_range(min_z..=max_z);

            while all_tree_transforms.clone().into_iter().any(|tr| {
                distance_between_vec3(
                    tr.translation,
                    Vec3 {
                        x: pos_x,
                        y: GROUND_Y + 1.0,
                        z: pos_z,
                    },
                ) <= MIN_GAP_BETWEEN_TREES
            }) {
                pos_x = rng.gen_range(min_x..=max_x);
                pos_z = rng.gen_range(min_z..=max_z);
            }
            // let transform = Transform::from_xyz(pos_x, GROUND_Y, pos_z);
            let tree = Tree::new(pos_x, pos_z);
            let (mesh, color) = tree.mesh_and_color();
            all_tree_transforms.push(tree.transform.clone());
            let bundle = PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(color),
                transform: tree.transform,
                ..Default::default() // transform:
            };
            commands.spawn((bundle, tree));
        }
    }
}

fn round_to_nearest_multiple(value: f64, multiple: f64) -> f64 {
    let scaled_value = value * (1.0 / multiple);

    let rounded_value = scaled_value.round();

    rounded_value * multiple
}

impl ChunkType {
    // bad
    fn color(&self) -> Color {
        match self {
            ChunkType::Lake => Color::from(BLUE),
            ChunkType::Field => Color::from(YELLOW),
            ChunkType::Forest => Color::from(GREEN),
        }
    }
}

#[derive(Component, Debug)]
struct Tree {
    height: f32,
    radius: f32,
    transform: Transform,
}

const TREE_HEIGHT_LOWEST: f32 = 20.0;
const TREE_HEIGHT_HIGHEST: f32 = 70.0;
const TREE_RADIUS_LOWEST: f32 = 0.2;
const TREE_RADIUS_HIGHEST: f32 = 0.7;

const MIN_GAP_BETWEEN_TREES: f32 = 2.0;

impl Tree {
    fn new(x: f32, z: f32) -> Self {
        let mut rng = rand::thread_rng();
        let height = rng.gen_range(TREE_HEIGHT_LOWEST..TREE_HEIGHT_HIGHEST);
        // idk dude, i though height/2 would be fine but i needed to move it down a little
        let transform = Transform::from_xyz(x, GROUND_Y + height / 2. - 4., z);
        Tree {
            height,
            radius: rng.gen_range(TREE_RADIUS_LOWEST..TREE_RADIUS_HIGHEST),
            transform,
        }
    }

    fn mesh_and_color(&self) -> (Mesh, Color) {
        let mesh = Mesh::from(Cylinder::new(self.radius, self.height));
        let material = Color::from(BROWN);
        (mesh, material)
    }
}

mod tests {
    use std::collections::HashMap;

    use bevy::math::Vec3;

    use crate::world::chunks::x_and_y_from_pos;

    #[test]
    fn chunk_pos_correct() {
        let (x, z) = (50., 105.);
        let pos = Vec3::new(x, 0., z);
        assert_eq!((1, 3), x_and_y_from_pos(&pos));
    }

    // fn collapse_edge(
    //     vertices: &mut Vec<[f32; 3]>,
    //     triangles: &mut Vec<[u32; 3]>,
    //     edge: (u32, u32),
    // ) {
    //     let (v1, v2) = edge;
    //     let new_vertex = [
    //         (vertices[v1 as usize][0] + vertices[v2 as usize][0]) / 2.0,
    //         (vertices[v1 as usize][1] + vertices[v2 as usize][1]) / 2.0,
    //         (vertices[v1 as usize][2] + vertices[v2 as usize][2]) / 2.0,
    //     ];
    //
    //     // Replace vertex v2 with v1
    //     for triangle in triangles.iter_mut() {
    //         if triangle[0] == v2 {
    //             triangle[0] = v1;
    //         } else if triangle[1] == v2 {
    //             triangle[1] = v1;
    //         } else if triangle[2] == v2 {
    //             triangle[2] = v1;
    //         }
    //     }
    //
    //     // Remove duplicate vertices
    //     let mut vertex_map = HashMap::new();
    //     let mut new_vertices = Vec::new();
    //     for vertex in vertices.iter() {
    //         let key = (vertex[0], vertex[1], vertex[2]);
    //         if !vertex_map.contains_key(&key) {
    //             vertex_map.insert(key, new_vertices.len() as u32);
    //             new_vertices.push(*vertex);
    //         }
    //     }
    //
    //     *vertices = new_vertices;
    //
    //     // Update vertex indices in triangles
    //     for triangle in triangles.iter_mut() {
    //         triangle[0] = *vertex_map
    //             .get(&(
    //                 vertices[triangle[0] as usize][0],
    //                 vertices[triangle[0] as usize][1],
    //                 vertices[triangle[0] as usize][2],
    //             ))
    //             .unwrap();
    //         triangle[1] = *vertex_map
    //             .get(&(
    //                 vertices[triangle[1] as usize][0],
    //                 vertices[triangle[1] as usize][1],
    //                 vertices[triangle[1] as usize][2],
    //             ))
    //             .unwrap();
    //         triangle[2] = *vertex_map
    //             .get(&(
    //                 vertices[triangle[2] as usize][0],
    //                 vertices[triangle[2] as usize][1],
    //                 vertices[triangle[2] as usize][2],
    //             ))
    //             .unwrap();
    //     }
    // }
}
