use bevy::{color::palettes::css::GREEN, prelude::*, render::primitives::Sphere};
use bevy_rapier3d::prelude::*;
use noise::{Fbm, Perlin};

use crate::{rtin::build_terrain_from_noise, world::GROUND_Y};

use super::chunks::{Chunk, ChunkMap, ChunkType};

#[derive(Component)]
struct TerrainChunk;

pub fn is_power_of_2(x: u32) -> bool {
    (x & !(x & (x - 1))) > 0
}

pub fn spawn_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut noise_func = Fbm::<Perlin>::new(5);
    noise_func.lacunarity = 0.2;
    noise_func.frequency = 0.0125;
    noise_func.octaves = 2;
    noise_func.persistence = 0.2;

    let transform = Transform::from_xyz(0., GROUND_Y, 0.);
    let size = 256.;
    let err_threshold = 0.01;
    let height_multiplier = 50.;

    assert!(is_power_of_2(size as u32));
    let terrain = build_terrain_from_noise(&mut noise_func, height_multiplier, size, err_threshold);
    let mesh = terrain.into_mesh(false, size);
    let collider = Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).unwrap();

    let bundle = PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::from(GREEN)),
        transform,
        ..Default::default()
    };

    let name = Name::new("terrain");

    let mut com = commands.spawn((bundle, name, TerrainChunk));
    com.with_children(|p| {
        p.spawn(collider)
            .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)));
    });
}

pub fn spawn_terrain_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<ChunkMap>,
) {
    for (_, chunk) in map.as_ref().0.iter() {
        if let ChunkType::Forest = chunk.typ {
            chunk.spawn_trees(&mut commands, &mut meshes, &mut materials);
        }
    }
}
