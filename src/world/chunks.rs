use std::{ops::Range, sync::LazyLock};

use crate::player::Player;

use super::GROUND_Y;
use bevy::{
    color::palettes::css::{BLUE, BROWN, GREEN, WHITE, YELLOW},
    prelude::*,
    reflect::List,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
    transform::commands,
    utils::{tracing::instrument, HashMap},
};
use bevy_rapier3d::{
    na::{distance_squared, DMatrix},
    parry::math::Point,
    prelude::*,
    rapier::prelude::HeightField,
};
use noise::NoiseFn;
use noise::{Fbm, Perlin};
use rand::{distributions::uniform::SampleRange, prelude, Rng};

#[derive(Debug, Clone, PartialEq, Eq)]
enum ChunkType {
    Lake,
    Field,
    Forest,
}

#[derive(Debug)]
pub struct Chunk {
    typ: ChunkType,
    // coordinates of the bottom left corner of the chunk
    x: f32,
    z: f32,
}

// Simply a grid of chunks mapped over the 3D space
// The hash IS NOT their coordinates in space, but their position on the grid
#[derive(Resource, Debug)]
pub struct ChunkMap(HashMap<(usize, usize), Chunk>);

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

pub fn render_chunks_near_player(
    mut commands: Commands,
    map: Res<ChunkMap>,
    player_q: Query<&Transform, With<Player>>,
) {
    let acceptable_distance = CHUNK_SIZE * 3.;

    let transform = player_q.single();

    // for (_, chunk) in map.as_ref().0.iter() {
    //     if chunk.x - transform.translation.x > acceptable_distance &&
    //  chunk.z - transform.translation.z > acceptable_distance {
    //     // commands.entity(chunk)
    // }
    // }
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
    fn x_range(&self) -> Range<i32> {
        self.x as i32..self.x as i32 + CHUNK_SIZE as i32
    }
    fn z_range(&self) -> Range<i32> {
        self.z as i32..self.z as i32 + CHUNK_SIZE as i32
    }

    fn mesh(&self, noise: &mut Fbm<Perlin>) -> (Mesh, Collider) {
        let y_scaling: f64 = 50.0;
        let water_level: f64 = 20.0;

        let num_vertices = CHUNK_SIZE as usize;

        let mut vertices = Vec::with_capacity(num_vertices * num_vertices);
        let mut heights = vec![0.; num_vertices * num_vertices];
        let mut indices = Vec::with_capacity(num_vertices * num_vertices * 6);
        let mut uvs = Vec::with_capacity(num_vertices * num_vertices);

        for z in 0..num_vertices {
            for x in 0..num_vertices {
                let noise_x = x as f64 + self.x as f64;
                let noise_z = z as f64 + self.z as f64;

                let mut y_pos = noise.get([noise_x, noise_z]) * y_scaling;
                y_pos = round_to_nearest_multiple(y_pos, 5.);

                vertices.push([x as f32, y_pos as f32, z as f32]);
                uvs.push([x as f32 / CHUNK_SIZE, z as f32 / CHUNK_SIZE]);
                // let y_pos = noise.get([noise_x, noise_z]) * y_scaling;
                heights[x * num_vertices + z] = y_pos as f32;

                if x < num_vertices - 1 && z < num_vertices - 1 {
                    let idx = x + z * num_vertices;
                    indices.push(idx as u32);
                    indices.push((idx + num_vertices) as u32);
                    indices.push((idx + 1) as u32);
                    indices.push((idx + 1) as u32);
                    indices.push((idx + num_vertices) as u32);
                    indices.push((idx + num_vertices + 1) as u32);
                }
            }
        }

        let field = Collider::heightfield(
            heights,
            num_vertices,
            num_vertices,
            Vec3 {
                x: CHUNK_SIZE - 1.0,
                y: 1.0,
                z: CHUNK_SIZE - 1.0,
            },
        );
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));
        mesh.compute_normals();

        (mesh, field)
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

pub fn spawn_terrain(
    mut commands: Commands,
    map: Res<ChunkMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut noise_func = Fbm::<Perlin>::new(5);
    noise_func.lacunarity = 0.2;
    noise_func.frequency = 0.025;
    noise_func.octaves = 1;
    noise_func.persistence = 0.2;

    info!("starting terrain rendering");

    for ((i, k), chunk) in map.as_ref().0.iter() {
        info!("creating bundle");

        let transform = Transform::from_xyz(chunk.x, GROUND_Y, chunk.z);
        let (mesh, collider) = chunk.mesh(&mut noise_func);

        let bundle = PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(chunk.typ.color()),
            transform,
            ..Default::default()
        };

        let name = Name::new(format!("chunk ({},{})", i, k));

        // commands
        //     .spawn((Ground,))
        //     .insert(TransformBundle::from(Transform::from_xyz(x, -2.0, z)));

        let mut com = commands.spawn((bundle, name));

        if chunk.typ != ChunkType::Lake {
            com.with_children(|p| {
                p.spawn(collider)
                    .insert(TransformBundle::from(Transform::from_xyz(
                        CHUNK_SIZE / 2. - 0.5,
                        0.0,
                        CHUNK_SIZE / 2. - 0.5,
                    )));
            });
        }
    }
}

pub fn update_terrain_materials(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // mut floor_q: Query<&mut PbrBundle, With<Floor>>,
) {
    // let water_level = -1.0;
    // let mountain_level = 20.0;
    // for floor in floor_q.iter_mut() {
    //     floor
    // }
}

pub fn spawn_terrain_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<ChunkMap>,
) {
    let mut rng = rand::thread_rng();
    for (_, chunk) in map.as_ref().0.iter() {
        if let ChunkType::Forest = chunk.typ {
            let amt_of_trees = 25;
            let mut all_tree_transforms: Vec<Transform> = vec![];
            for _ in 0..=amt_of_trees {
                // let (x, y, z) = (
                //     trans.translation.x,
                //     trans.translation.y,
                //     trans.translation.z,
                // );

                // add some margins
                let min_x = chunk.x_range().start as f32 + MIN_GAP_BETWEEN_TREES;
                let min_z = chunk.z_range().start as f32 + MIN_GAP_BETWEEN_TREES;
                let max_x = chunk.x_range().end as f32 - MIN_GAP_BETWEEN_TREES;
                let max_z = chunk.z_range().end as f32 - MIN_GAP_BETWEEN_TREES;

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
}
mod tests {
    use bevy::math::Vec3;

    use crate::world::chunks::x_and_y_from_pos;

    #[test]
    fn chunk_pos_correct() {
        let (x, z) = (50., 105.);
        let pos = Vec3::new(x, 0., z);
        assert_eq!((1, 3), x_and_y_from_pos(&pos));
    }
}
