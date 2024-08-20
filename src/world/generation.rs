use std::{ops::Range, sync::LazyLock};

use super::GROUND_Y;
use bevy::{
    color::palettes::css::{BLUE, BROWN, GREEN, YELLOW},
    prelude::*,
    reflect::List,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
    transform::commands,
};
use bevy_rapier3d::{na::distance_squared, prelude::*};
use noise::NoiseFn;
use noise::{Fbm, Perlin};
use rand::{distributions::uniform::SampleRange, prelude, Rng};

#[derive(Component)]
pub enum WorldTile {
    Lake,
    Field,
    Forest,
}

impl WorldTile {
    // bad
    fn color(&self) -> Color {
        match self {
            Self::Lake => Color::from(BLUE),
            Self::Field => Color::from(YELLOW),
            Self::Forest => Color::from(GREEN),
        }
    }

    fn mesh(&self) -> Mesh {
        let noise_func = Fbm::<Perlin>::new(0);
        let y_scaling: f64 = 5.0;
        let num_vertices = CHUNK_SIZE as usize;

        let mut vertices = Vec::with_capacity(num_vertices * num_vertices);
        let mut indices = Vec::with_capacity(num_vertices * num_vertices * 6);
        let mut uvs = Vec::with_capacity(num_vertices * num_vertices);

        for z in 0..num_vertices {
            for x in 0..num_vertices {
                // let x_pos = x as f32 - CHUNK_SIZE;
                // let z_pos = z as f32 - CHUNK_SIZE;
                let y_pos = noise_func.get([x as f64, z as f64]) * y_scaling;

                vertices.push([x as f32, y_pos as f32, z as f32]);
                uvs.push([x as f32 / CHUNK_SIZE, z as f32 / CHUNK_SIZE]);

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

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));
        mesh.compute_normals();

        mesh
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

        // let bundle = PbrBundle {
        //     mesh: meshes.add(mesh),
        //     material: materials.add(material),
        //     transform: Transform::from_xyz(x, y, z),
        //     ..Default::default()
        // };
    }
}

pub fn spawn_terrain_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    terrain_q: Query<(&WorldTile, &Transform)>,
) {
    let mut rng = rand::thread_rng();
    for (tile, trans) in terrain_q.iter() {
        if let WorldTile::Forest = tile {
            let amt_of_trees = 25;
            let mut all_tree_transforms: Vec<Transform> = vec![];
            for _ in 0..=amt_of_trees {
                let (x, y, z) = (
                    trans.translation.x,
                    trans.translation.y,
                    trans.translation.z,
                );

                // add some margins

                let min_x = x + MIN_GAP_BETWEEN_TREES;
                let min_z = z + MIN_GAP_BETWEEN_TREES;
                let max_x = x + (CHUNK_SIZE - MIN_GAP_BETWEEN_TREES);
                let max_z = z + (CHUNK_SIZE - MIN_GAP_BETWEEN_TREES);

                let mut pos_x = rng.gen_range(min_x..=max_x);
                let mut pos_z = rng.gen_range(min_z..=max_z);

                while all_tree_transforms.clone().into_iter().any(|tr| {
                    distance_between_vec3(
                        tr.translation,
                        Vec3 {
                            x: pos_x,
                            y,
                            z: pos_z,
                        },
                    ) <= MIN_GAP_BETWEEN_TREES
                }) {
                    pos_x = rng.gen_range(x..max_x);
                    pos_z = rng.gen_range(z..max_z);
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

fn distance_between_vec3(vec: Vec3, other: Vec3) -> f32 {
    let x = vec.x - other.x;
    let x = x * x;
    let z = vec.z - other.z;
    let z = z * z;
    (x + z).sqrt()
}

const CHUNK_SIZE: f32 = 50.0;
pub fn spawn_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (i, t) in [WorldTile::Forest, WorldTile::Field, WorldTile::Lake]
        .into_iter()
        .enumerate()
    {
        let terrain = PbrBundle {
            mesh: meshes.add(t.mesh()),
            material: materials.add(t.color()),
            // Have to subtract one, otherwise the chunks have space between them
            transform: Transform::from_xyz(i as f32 * (CHUNK_SIZE - 1.), GROUND_Y - 1.5, 0.0),
            ..Default::default()
        };
        commands.spawn((terrain, Name::new("terrain"), t));
    }
}

// pub fn spawn_terrain(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     let noise_func = Fbm::<Perlin>::new(0);
//     let y_scaling: f64 = 1.0;
//     let terrain_size = 20.0;
//     let num_vertices = (terrain_size * 2.0) as usize;
//
//     let mut vertices = Vec::with_capacity(num_vertices * num_vertices);
//     let mut indices = Vec::with_capacity(num_vertices * num_vertices * 6);
//     let mut uvs = Vec::with_capacity(num_vertices * num_vertices);
//
//     for z in 0..num_vertices {
//         for x in 0..num_vertices {
//             let x_pos = x as f32 - terrain_size;
//             let z_pos = z as f32 - terrain_size;
//             let y_pos = noise_func.get([x_pos as f64, z_pos as f64]) * y_scaling;
//
//             vertices.push([x_pos, y_pos as f32, z_pos]);
//             uvs.push([x as f32 / terrain_size, z as f32 / terrain_size]);
//
//             if x < num_vertices - 1 && z < num_vertices - 1 {
//                 let idx = x + z * num_vertices;
//                 indices.push(idx as u32);
//                 indices.push((idx + num_vertices) as u32);
//                 indices.push((idx + 1) as u32);
//                 indices.push((idx + 1) as u32);
//                 indices.push((idx + num_vertices) as u32);
//                 indices.push((idx + num_vertices + 1) as u32);
//             }
//         }
//     }
//
//     let mut mesh = Mesh::new(
//         PrimitiveTopology::TriangleList,
//         RenderAssetUsages::default(),
//     );
//     mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
//     mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
//     mesh.insert_indices(Indices::U32(indices));
//     mesh.compute_normals();
//
//     let terrain = PbrBundle {
//         mesh: meshes.add(mesh),
//         material: materials.add(Color::WHITE),
//         transform: Transform::from_xyz(0.0, GROUND_Y - 0.5, 0.0),
//         ..Default::default()
//     };
//     commands.spawn((terrain, Name::new("terrain")));
// }
