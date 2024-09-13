use std::{cmp::Ordering, sync::LazyLock};

use crate::{
    rtin::{build_terrain_from_sampler, noise::NoiseSampler},
    world::terrain::is_power_of_2,
};

use super::{
    heap::{Heapable, MinHeapMap},
    terrain::{Terrain, TerrainBundle, WORLD_COLLISION_GROUPS},
    wave::{
        Rotation, TileCellGrid, TileID, CORNER_0, CORNER_180, CORNER_270, CORNER_90, FLOOR, WALL_0,
        WALL_180, WALL_270, WALL_90,
    },
    GROUND_Y,
};
use bevy::{
    color::palettes::css::{BLACK, DARK_SLATE_GRAY, DIM_GREY, GREEN},
    prelude::{Transform, *},
    reflect::{List, Map},
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
    utils::{hashbrown::HashMap, HashSet},
};
use bevy_rapier3d::prelude::*;
use noise::{Fbm, Perlin};
use rand::{thread_rng, Rng};

const CHUNK_SIZE_XYZ: LazyLock<f32> =
    LazyLock::new(|| is_power_of_2(64.0).expect("ID PREFER THIS FOLLOW THIS RULE"));

impl TerrainBundle {
    fn new_chunk(
        mesh: Mesh,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Self {
        let collider = Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).unwrap();
        let mesh = meshes.add(mesh);
        let material = materials.add(StandardMaterial {
            base_color: BLACK.into(),
            metallic: 0.,
            reflectance: 1.0,
            ..Default::default()
        });

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

fn floor_mesh(meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) {
    let mut noise_func = Fbm::<Perlin>::new(5);
    noise_func.lacunarity = 0.2;
    noise_func.frequency = 0.0125;
    noise_func.octaves = 2;
    noise_func.persistence = 0.2;
    let size = *LazyLock::force(&CHUNK_SIZE_XYZ);
    let err_threshold = 0.01;
    let height_multiplier = 50.;

    let sampler = NoiseSampler::single_layer(noise_func);
    let terrain = build_terrain_from_sampler(&sampler, height_multiplier, size, err_threshold);
    let mesh = terrain.into_mesh(false, size);

    let bundle = TerrainBundle::new(mesh, meshes, materials);
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum MarchingTileType {
    Wall,
    Corner,
    // Ramp,
    Floor,
}

#[derive(Component, Debug, Clone)]
pub struct MarchingTile {
    rotation: Rotation,
    typ: MarchingTileType,
}

impl From<TileID> for MarchingTileType {
    fn from(tile_id: TileID) -> Self {
        match tile_id {
            _ if tile_id == WALL_0
                || tile_id == WALL_90
                || tile_id == WALL_180
                || tile_id == WALL_270 =>
            {
                Self::Wall
            }
            _ if tile_id == CORNER_0
                || tile_id == CORNER_90
                || tile_id == CORNER_180
                || tile_id == CORNER_270 =>
            {
                Self::Corner
            }

            _ if tile_id == FLOOR => Self::Floor,
            other => panic!("{other} is an invalid tile id"),
        }
    }
}

#[derive(Bundle)]
pub struct MarchingTileBundle {
    item: MarchingTile,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    collider: Collider,
    collision_groups: CollisionGroups,
    restitution: Restitution,
    friction: Friction,
    rigid_body: RigidBody,
    transform: TransformBundle,
    visibility: VisibilityBundle,
}

impl MarchingTileBundle {
    pub fn marching_tile_mesh(
        origin: &Transform,
        tile: &MarchingTile,
        x: u32,
        z: u32,
    ) -> (Mesh, Transform) {
        let size = *LazyLock::force(&CHUNK_SIZE_XYZ);
        let mesh = match tile.typ {
            MarchingTileType::Floor => Self::floor_mesh(),
            MarchingTileType::Corner => Self::corner_mesh(),
            MarchingTileType::Wall => Self::wall_mesh(),
        };
        let x_pos = (origin.translation.x + (size / 2.)) * x as f32;
        let z_pos = (origin.translation.z + (size / 2.)) * z as f32;

        let transform = Transform::from_xyz(x_pos, origin.translation.y, z_pos);
        warn!("tile transform: {transform:?}");
        (mesh.rotated_by(*tile.rotation.as_ref()), transform)
    }

    pub fn wall_mesh() -> Mesh {
        let size = *LazyLock::force(&CHUNK_SIZE_XYZ);
        // 6 planes on a cuboid
        let plane_width = size / 6.0 as f32;
        let plane_depth = 1.0;

        let cuboid = Cuboid::new(plane_width, plane_width, plane_depth);

        let mut mesh: Mesh = cuboid.mesh().into();
        // mesh.translate_by(Vec3::new(-size / 2., size / 2., 0.));
        mesh
    }

    fn floor_mesh() -> Mesh {
        let mesh = Self::wall_mesh();

        let floor_rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        mesh.rotated_by(floor_rotation)
    }

    fn ramp_mesh() -> Mesh {
        let mesh = Self::wall_mesh();
        let rotation_30_degrees = Quat::from_rotation_x(std::f32::consts::FRAC_PI_3); // 30 degrees in radians
        mesh.rotated_by(rotation_30_degrees)
    }

    /// Corner created by one regular sized wall, and another wall width -1, the other wall is
    /// rotated to match with the other.
    fn corner_mesh() -> Mesh {
        let size = *LazyLock::force(&CHUNK_SIZE_XYZ);
        // 6 planes on a cuboid
        let plane_width = size / 6.0 as f32;
        let plane_depth = 1.0;

        let mut larger_wall = Mesh::from(Cuboid::new(plane_width, plane_width, plane_depth));

        let rotation = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2);
        let x = (plane_width / 2.0) - 0.5;
        let z = ((plane_width - 1.0) / 2.0) + 0.5;

        let translation = Vec3::new(x, 0.0, z);
        let mut smaller_wall = Mesh::from(Cuboid::new(plane_width - 1.0, plane_width, plane_depth));
        smaller_wall.rotate_by(rotation);
        smaller_wall.translate_by(translation);
        larger_wall.merge(&smaller_wall);
        larger_wall
    }
}

pub const TILE_ID_MARCHING_TILES_MAP: LazyLock<HashMap<TileID, MarchingTile>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        map.insert(
            FLOOR,
            MarchingTile {
                rotation: 0.into(),
                typ: MarchingTileType::Floor,
            },
        );
        map.insert(
            WALL_0,
            MarchingTile {
                rotation: 0.into(),
                typ: MarchingTileType::Wall,
            },
        );
        map.insert(
            WALL_90,
            MarchingTile {
                rotation: 90.into(),
                typ: MarchingTileType::Wall,
            },
        );
        map.insert(
            WALL_180,
            MarchingTile {
                rotation: 180.into(),
                typ: MarchingTileType::Wall,
            },
        );
        map.insert(
            WALL_270,
            MarchingTile {
                rotation: 270.into(),
                typ: MarchingTileType::Wall,
            },
        );
        map.insert(
            CORNER_0,
            MarchingTile {
                rotation: 0.into(),
                typ: MarchingTileType::Corner,
            },
        );
        map.insert(
            CORNER_90,
            MarchingTile {
                rotation: 90.into(),
                typ: MarchingTileType::Corner,
            },
        );
        map.insert(
            CORNER_180,
            MarchingTile {
                rotation: 180.into(),
                typ: MarchingTileType::Corner,
            },
        );
        map.insert(
            CORNER_270,
            MarchingTile {
                rotation: 270.into(),
                typ: MarchingTileType::Corner,
            },
        );
        map
    });
