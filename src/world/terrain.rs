use bevy::{color::palettes::css::GREEN, prelude::*};
use bevy_rapier3d::prelude::*;
use noise::{Fbm, Perlin};
use std::sync::LazyLock;

use crate::{
    rtin::{build_terrain_from_sampler, noise::NoiseSampler},
    world::GROUND_Y,
};

use super::chunks::{ChunkMap, ChunkType};

#[derive(Component)]
pub struct Terrain;

pub const WORLD_COLLISION_GROUPS: LazyLock<CollisionGroups> =
    LazyLock::new(|| CollisionGroups::new(Group::GROUP_1, Group::GROUP_2 | Group::GROUP_3));

#[derive(Bundle)]
pub struct TerrainBundle {
    pub(super) terrain: Terrain,
    pub(super) name: Name,
    pub(super) collider: Collider,
    pub(super) rigid_body: RigidBody,
    pub(super) collision_groups: CollisionGroups,
    pub(super) transform: TransformBundle,
    pub(super) visibility: VisibilityBundle,
    pub(super) mesh: Handle<Mesh>,
    pub(super) material: Handle<StandardMaterial>,
}

impl TerrainBundle {
    pub(super) fn new(
        mesh: Mesh,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Self {
        let collider = Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).unwrap();
        let mesh = meshes.add(mesh);
        let material = materials.add(Color::from(GREEN));

        let transform = Transform::from_xyz(0., GROUND_Y, 0.);
        Self {
            terrain: Terrain,
            name: Name::new("Terrain"),
            collider,
            rigid_body: RigidBody::Fixed,
            collision_groups: LazyLock::force(&WORLD_COLLISION_GROUPS).to_owned(),
            transform: TransformBundle::from_transform(transform),
            visibility: VisibilityBundle::default(),
            mesh,
            material,
        }
    }
}

pub fn is_power_of_2(x: f32) -> Option<f32> {
    let x = x as u32;
    if (x & !(x & (x - 1))) > 0 {
        Some(x as f32)
    } else {
        None
    }
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
    let size = 256.;
    let err_threshold = 0.01;
    let height_multiplier = 50.;

    let sampler = NoiseSampler::single_layer(noise_func);

    assert!(is_power_of_2(size).is_some());
    let terrain = build_terrain_from_sampler(&sampler, height_multiplier, size, err_threshold);
    let mesh = terrain.into_mesh(false, size);

    let bundle = TerrainBundle::new(mesh, &mut meshes, &mut materials);
    commands.spawn(bundle);
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
