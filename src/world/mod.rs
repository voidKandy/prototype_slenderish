mod atmosphere;
pub mod chunks;
pub mod noise;
pub mod rtin;
pub mod terrain;
pub mod wfc;
use atmosphere::SkyMaterial;
use bevy::{color::palettes::css::YELLOW, math::NormedVectorSpace, prelude::*};
use rtin::TerrainMeshData;
use terrain::spawn_terrain;
pub use wfc::heap_map::Heapable;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SkyMaterial>::default())
            .add_systems(
                Startup,
                (atmosphere::setup_atmosphere, spawn_terrain, spawn_light).chain(),
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

    TerrainMeshData { vertices, indices }
}
