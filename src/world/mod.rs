mod chunks;
pub mod terrain;
use std::{collections::HashMap, f32::consts::PI};

use bevy::{
    color::palettes::css::{RED, WHITE, YELLOW},
    math::NormedVectorSpace,
    prelude::{Cuboid, *},
    reflect::List,
    render::{
        mesh::{Indices, PrimitiveTopology, SphereKind, SphereMeshBuilder},
        render_asset::RenderAssetUsages,
    },
};
use chunks::setup_chunk_map;
use terrain::{spawn_terrain, spawn_terrain_entities};

use crate::rtin::TerrainMeshData;
// use tiles::{spawn_terrain, spawn_terrain_entities};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                setup_chunk_map,
                spawn_skymap,
                spawn_terrain,
                spawn_terrain_entities,
                spawn_objects,
                spawn_light,
            )
                .chain(),
        );
    }
}

pub const GROUND_Y: f32 = 0.0;

fn spawn_light(mut commands: Commands) {
    let light = PointLightBundle {
        point_light: PointLight {
            intensity: 20000.0,
            shadows_enabled: true,
            color: YELLOW.into(),
            radius: 600.0,
            range: 600.,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 10.0, 0.0),
        ..Default::default()
    };
    commands.spawn((light, Name::new("main light")));
}

fn spawn_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let blue_cube = PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid::new(0.5, 0.5, 0.5))),
        material: materials.add(Color::srgb(0., 0., 0.5)),
        transform: Transform::from_xyz(0., -1.5, 0.),
        ..Default::default()
    };

    let red_cube = PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid::new(8.0, 8.0, 8.0))),
        material: materials.add(Color::srgb(0.5, 0., 0.)),
        transform: Transform::from_xyz(-4., GROUND_Y, 8.),
        ..Default::default()
    };

    commands.spawn((Name::new("red cube"), red_cube));
    commands.spawn((Name::new("blue cube"), blue_cube));
}

pub fn spawn_skymap(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sphere = PbrBundle {
        mesh: meshes.add(icosahedron(0).into_mesh(true, 5.)),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            metallic: 1.0,
            perceptual_roughness: 0.,
            reflectance: 1.,
            ..Default::default()
        }),
        transform: Transform::from_xyz(10., 2., 0.),
        ..Default::default()
    };
    commands.spawn(sphere);
}

fn project_to_unit_sphere(vertices: &mut Vec<Vec3>) {
    for mut v in vertices.iter_mut() {
        let n = v.norm();
        *v = (1.0 / n) * *v;
    }
}

fn icosahedron(recursion_level: usize) -> TerrainMeshData {
    let phi = (1.0 + 5.0f32.sqrt()) * 0.5; // golden ratio

    // let vertices = vec![
    //     Vec3::new(-1.0, phi, 0.0),
    //     Vec3::new(1.0, phi, 0.0),
    //     Vec3::new(-1.0, -phi, 0.0),
    //     Vec3::new(1.0, -phi, 0.0),
    //     Vec3::new(0.0, -1.0, phi),
    //     Vec3::new(0.0, 1.0, phi),
    //     Vec3::new(0.0, -1.0, -phi),
    //     Vec3::new(0.0, 1.0, -phi),
    //     Vec3::new(phi, 0.0, -1.0),
    //     Vec3::new(phi, 0.0, 1.0),
    //     Vec3::new(-phi, 0.0, -1.0),
    //     Vec3::new(-phi, 0.0, 1.0),
    // ];
    let vertices = vec![
        Vec3::new(1.0, -phi, 0.0),  // Negated (-1.0, phi, 0.0)
        Vec3::new(-1.0, -phi, 0.0), // Negated (1.0, phi, 0.0)
        Vec3::new(1.0, phi, 0.0),   // Negated (-1.0, -phi, 0.0)
        Vec3::new(-1.0, phi, 0.0),  // Negated (1.0, -phi, 0.0)
        Vec3::new(0.0, 1.0, -phi),  // Negated (0.0, -1.0, phi)
        Vec3::new(0.0, -1.0, -phi), // Negated (0.0, 1.0, phi)
        Vec3::new(0.0, 1.0, phi),   // Negated (0.0, -1.0, -phi)
        Vec3::new(0.0, -1.0, phi),  // Negated (0.0, 1.0, -phi)
        Vec3::new(-phi, 0.0, 1.0),  // Negated (phi, 0.0, -1.0)
        Vec3::new(-phi, 0.0, -1.0), // Negated (phi, 0.0, 1.0)
        Vec3::new(phi, 0.0, 1.0),   // Negated (-phi, 0.0, -1.0)
        Vec3::new(phi, 0.0, -1.0),  // Negated (-phi, 0.0, 1.0)
    ];

    let indices = vec![
        0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11, 1, 5, 9, 5, 11, 4, 11, 10, 2, 10, 7, 6, 7,
        1, 8, 3, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9, 4, 9, 5, 2, 4, 11, 6, 2, 10, 8, 6, 7, 9,
        8, 1,
    ];

    // let mut idx_map = HashMap::<i64, f32>::new();
    // let mut faces2 = Vec::new();
    // for _ in 0..recursion_level {
    //     for window in indices.windows(3) {
    //         let (x, y, z) = (window[0], window[1], window[2]);
    //         let a = get_middle_point(&mut idx_map, x, y, &mut vertices);
    //         let b = get_middle_point(&mut idx_map, y, z, &mut vertices);
    //         let c = get_middle_point(&mut idx_map, z, x, &mut vertices);
    //
    //         vertices.push(Vec3::new(x, a, c));
    //         vertices.push(Vec3::new(y, b, a));
    //         vertices.push(Vec3::new(z, c, b));
    //         vertices.push(Vec3::new(a, b, c));
    //
    //         // faces2.push(TriangleIndices { v1: tri.v1, v2: a, v3: c });
    //         // faces2.push(TriangleIndices { v1: tri.v2, v2: b, v3: a });
    //         // faces2.push(TriangleIndices { v1: tri.v3, v2: c, v3: b });
    //         // faces2.push(TriangleIndices { v1: a, v2: b, v3: c });
    //     }
    //     // faces = faces2;
    // }
    //

    TerrainMeshData { vertices, indices }
}

fn get_middle_point(
    middle_point_idx_map: &mut HashMap<i64, f32>,
    p1: usize,
    p2: usize,
    vertices: &mut Vec<Vec3>,
) -> f32 {
    let first_is_smaller = p1 < p2;
    let smaller_index = if first_is_smaller { p1 } else { p2 };
    let greater_index = if first_is_smaller { p2 } else { p1 };
    let key = ((smaller_index as i64) << 32) + greater_index as i64;

    if let Some(&ret) = middle_point_idx_map.get(&key) {
        return ret;
    }

    let point1 = vertices[p1];
    let point2 = vertices[p2];
    let middle = Vec3::new(
        (point1.x + point2.x) / 2.0,
        (point1.y + point2.y) / 2.0,
        (point1.z + point2.z) / 2.0,
    );

    vertices.push(middle);
    let i = vertices.len();

    // let i = self.add_vertex(middle);
    middle_point_idx_map.insert(key, 0f32);
    // idk lol
    0f32
}

//   fn  getMiddlePoint(p1: u32,p2:  u32) -> f32
//     {
//         // first check if we have it already
//         let first_smaller = p1 < p2;
//     let (smaller_idx, greater_idx)= if first_smaller {
//         (p1, p2)
// } else {
//         (p2, p1)
//
// };
//  let key = (smaller_idx << 32) + greater_idx;
//
//         if (this.middlePointIndexCache.TryGetValue(key, out ret))
//         {
//             return ret;
//         }
//
//         // not in cache, calculate it
//         Point3D point1 = this.geometry.Positions[p1];
//         Point3D point2 = this.geometry.Positions[p2];
//         Point3D middle = new Point3D(
//             (point1.X + point2.X) / 2.0,
//             (point1.Y + point2.Y) / 2.0,
//             (point1.Z + point2.Z) / 2.0);
//
//         // add vertex makes sure point is on unit sphere
//         int i = addVertex(middle);
//
//         // store it, return index
//         this.middlePointIndexCache.Add(key, i);
//         return i;
//     }
