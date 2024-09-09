use std::sync::LazyLock;

use bevy::{color::palettes::css::GREEN, prelude::*};
use bevy_rapier3d::prelude::*;
use noise::{Fbm, Perlin};

use crate::{rtin::build_terrain_from_noise, world::GROUND_Y};

use super::chunks::{ChunkMap, ChunkType};

#[derive(Component)]
struct Terrain;

const WORLD_COLLISION_GROUPS: LazyLock<CollisionGroups> =
    LazyLock::new(|| CollisionGroups::new(Group::GROUP_1, Group::GROUP_2 | Group::GROUP_3));

#[derive(Bundle)]
pub struct TerrainBundle {
    terrain: Terrain,
    name: Name,
    collider: Collider,
    rigid_body: RigidBody,
    collision_groups: CollisionGroups,
    transform: TransformBundle,
    visibility: VisibilityBundle,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

impl TerrainBundle {
    fn new(
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
    let size = 256.;
    let err_threshold = 0.01;
    let height_multiplier = 50.;

    assert!(is_power_of_2(size as u32));
    let terrain = build_terrain_from_noise(&mut noise_func, height_multiplier, size, err_threshold);
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
