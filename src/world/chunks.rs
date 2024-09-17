use crate::world::{noise::NoiseSampler, rtin::build_terrain_from_sampler, terrain::is_power_of_2};
use bevy::{color::palettes::css::BLACK, prelude::*};
use super::{
    terrain::{Terrain, TerrainBundle, WORLD_COLLISION_GROUPS},
    wfc::{grid::TileCell, tile::TileID},
    GROUND_Y,
};
use bevy_rapier3d::prelude::*;
use noise::{Fbm, Perlin};
use std::sync::LazyLock;

const CHUNK_SIZE_XYZ: LazyLock<f32> = LazyLock::new(|| {
    let size = 16.0;
    if is_power_of_2(size) {
        return size;
    }
    panic!("this needs to be a square");
});

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
    let terrain = build_terrain_from_sampler (&sampler, height_multiplier, size, err_threshold);
    let mesh = terrain.into_mesh(false, size);

    let bundle = TerrainBundle::new(mesh, meshes, materials);
}

#[derive(Component, Debug, Clone)]
pub struct MarchingTile {
    id: TileID,
}

impl From<TileID> for MarchingTile {
    fn from(id: TileID) -> Self {
        Self { id }
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
    /// Eventually should just return a marching tile bundle
    pub fn cell_mesh(cell: &TileCell) -> Option<Mesh> {
        let size = *LazyLock::force(&CHUNK_SIZE_XYZ);
        let mut mesh = match cell.id.type_value() {
            TileID::EMPTY => None,
            TileID::WALL => Some(Self::wall_mesh(size)),
            TileID::CORNER => Some(Self::corner_mesh(size)),
            _ => panic!("no mesh for {}", cell.id.to_string()),
        };

        let local = cell.local_transform(size);
        if let Some(m) = mesh.as_mut() {
            m.transform_by(local);
        }

        mesh
    }

    pub fn global_transform(cell: &TileCell, origin: &Transform) -> GlobalTransform {
        let size = *LazyLock::force(&CHUNK_SIZE_XYZ);
        assert!(is_power_of_2(size));
        // in order to translate to 3D space, we need to swap x and z
        let (z, x) = (cell.x - 1, cell.z - 1);
        warn!(
            "getting transform for {}\nx: {x}\nz: {z}",
            cell.id.to_string()
        );
        let x_pos = origin.translation.x - size * x as f32;
        let z_pos = origin.translation.z + size * z as f32;
        let t = Transform::from_xyz(x_pos, origin.translation.y, z_pos);
        warn!("got global transform: {:?}\n", t.translation);
        t.into()
    }

    pub fn wall_mesh(size: f32) -> Mesh {
        let plane_size_x = size;
        let plane_size_y = plane_size_x;
        let plane_size_z = 1.0;
        let cuboid = Cuboid::new(plane_size_x, plane_size_y, plane_size_z);
        let mesh: Mesh = cuboid.mesh().into();
        mesh
    }

    fn floor_mesh(size: f32) -> Mesh {
        let mesh = Self::wall_mesh(size);
        let floor_rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        mesh.rotated_by(floor_rotation)
    }

    // fn ramp_mesh() -> Mesh {
    //     let mesh = Self::wall_mesh();
    //     let rotation_30_degrees = Quat::from_rotation_x(std::f32::consts::FRAC_PI_3); // 30 degrees in radians
    //     mesh.rotated_by(rotation_30_degrees)
    // }

    /// Corner created by one regular sized wall, and another wall width -1, the other wall is
    /// rotated to match with the other.
    fn corner_mesh(size: f32) -> Mesh {
        let plane_size_x = size;
        let plane_size_y = plane_size_x;
        let plane_size_z = 1.0;

        let mut larger_wall = Mesh::from(Cuboid::new(plane_size_x, plane_size_y, plane_size_z));

        let rotation = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2);
        let x = (plane_size_x / 2.0) - 0.5;
        let z = ((plane_size_x - 1.0) / 2.0) + 0.5;

        let translation = Vec3::new(x, 0.0, z);
        let mut smaller_wall =
            Mesh::from(Cuboid::new(plane_size_x - 1.0, plane_size_y, plane_size_z));
        smaller_wall.rotate_by(rotation);
        smaller_wall.translate_by(translation);
        larger_wall.merge(&smaller_wall);
        larger_wall
    }
}
