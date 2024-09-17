#[path = "../bin/common/lib.rs"]
mod common;

use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use noise::{Fbm, Perlin};
use prototype_slenderish::{
    noise::NoiseSampler, rtin::build_terrain_from_sampler, world::terrain::TerrainBundle,
};

pub fn main() {
    let mut app = common::test_app();
    app.add_systems(Startup, setup_nosie)
        // .add_systems(Update, NoiseListener::update)
        .run();
}
#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct NoiseListener;

fn setup_nosie(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut noise_func = Fbm::<Perlin>::new(5);
    noise_func.lacunarity = 0.2;
    noise_func.frequency = 0.1;
    noise_func.octaves = 2;
    noise_func.persistence = 0.2;
    let sampler = NoiseSampler::single_layer(noise_func);
    let size = 128.;
    let height = 20.0;

    let terrain = build_terrain_from_sampler(&sampler, height, size, 0.01);
    let mesh = terrain.into_mesh(false, size);
    commands.insert_resource(sampler);

    let mut bundle = TerrainBundle::new(mesh, &mut meshes, &mut materials);
    let transform = Transform::from_xyz(-50., 0., -50.);
    bundle.transform = transform.into();
    commands.spawn(bundle);
}
//
//
// //! Shows how to sample noise on the CPU.
// //!
// //! Generates a simple fbm island height map on the CPU and spawns tiles
// //! with corresponding colors.
//
// use bevy::{math::vec2, prelude::*, render::camera::ScalingMode};
// use bevy_pancam::{PanCam, PanCamPlugin};
// use noisy_bevy::fbm_simplex_2d;
// fn setup(mut commands: Commands) {
//     let mut cam = Camera2dBundle::default();
//     cam.projection.scaling_mode = ScalingMode::FixedVertical(70.);
//     commands.spawn((cam, PanCam::default()));
//
//     const FREQUENCY_SCALE: f32 = 0.05;
//     const AMPLITUDE_SCALE: f32 = 4.0;
//     const RADIUS: f32 = 30.;
//     const OCTAVES: usize = 3;
//     const LACUNARITY: f32 = 2.;
//     const GAIN: f32 = 0.5;
//
//     let grid_half_size = RADIUS as i32 + 1;
//
//     for x in -grid_half_size..=grid_half_size {
//         for y in -grid_half_size..=grid_half_size {
//             let p = vec2(x as f32, y as f32);
//
//             // this is the whole point of the example
//             let offset =
//                 fbm_simplex_2d(p * FREQUENCY_SCALE, OCTAVES, LACUNARITY, GAIN) * AMPLITUDE_SCALE;
//
//             let height = RADIUS + offset - ((x * x + y * y) as f32).sqrt();
//
//             // spawn a corresponding tile with a color thats more white the higher the height
//             commands.spawn(SpriteBundle {
//                 sprite: Sprite {
//                     color: Color::WHITE.with_luminance(height * 0.03),
//                     custom_size: Some(Vec2::splat(1.)),
//                     ..default()
//                 },
//                 transform: Transform::from_translation(Vec3::new(x as f32, y as f32, 100.)),
//                 ..default()
//             });
//         }
//     }
// }
