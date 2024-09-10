pub(super) mod sphere;
use super::{EquipItem, EquipItemMaterial};
use bevy::{color::palettes::css::PURPLE, pbr::ExtendedMaterial, prelude::*};
use bevy_rapier3d::prelude::*;
use sphere::WorldSphereState;
use std::{default, sync::LazyLock};

pub const ITEM_COLLISION_GROUPS: LazyLock<CollisionGroups> = LazyLock::new(|| {
    CollisionGroups::new(
        Group::GROUP_3,
        Group::GROUP_2 | Group::GROUP_1 | Group::GROUP_3,
    )
});

#[derive(Component, Debug, Clone)]
pub enum WorldEquipItem {
    Sphere(WorldSphereState),
    Cube,
}

impl From<EquipItem> for WorldEquipItem {
    fn from(value: EquipItem) -> Self {
        match value {
            EquipItem::Cube => Self::Cube,
            EquipItem::Sphere => Self::Sphere(WorldSphereState::default()),
        }
    }
}

impl Into<EquipItem> for WorldEquipItem {
    fn into(self) -> EquipItem {
        match self {
            Self::Cube => EquipItem::Cube,
            Self::Sphere(_) => EquipItem::Sphere,
        }
    }
}

#[derive(Bundle)]
pub struct WorldEquipItemBundle {
    pub(super) item: WorldEquipItem,
    collider: Collider,
    collision_groups: CollisionGroups,
    restitution: Restitution,
    friction: Friction,
    rigid_body: RigidBody,
    mass_properties: AdditionalMassProperties,
    transform: TransformBundle,
    visibility: VisibilityBundle,
    pub(super) mesh: Handle<Mesh>,
    pub(super) material: Handle<EquipItemMaterial>,
}

impl WorldEquipItemBundle {
    pub fn from_equip_item(
        item: EquipItem,
        transform: Transform,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<EquipItemMaterial>>,
    ) -> Self {
        let mf = |size| MassProperties {
            mass: size * 10.,
            ..Default::default()
        };
        let (mesh, material, collider, mass) = match item {
            EquipItem::Cube => {
                let size = 5.;
                let mesh: Mesh = Cuboid::new(size, size, size).into();
                let material = ExtendedMaterial {
                    base: StandardMaterial {
                        base_color: PURPLE.into(),
                        opaque_render_method: bevy::pbr::OpaqueRendererMethod::Auto,
                        ..Default::default()
                    },
                    extension: crate::player::PlayerViewModelExtension { quantize_steps: 3 },
                };

                let collider_size = size / 2.;

                let collider = Collider::cuboid(collider_size, collider_size, collider_size);
                (mesh, material, collider, mf(size))
            }
            EquipItem::Sphere => {
                let size = 0.2;
                let mesh: Mesh = Sphere::new(size).into();
                let material = ExtendedMaterial {
                    base: StandardMaterial {
                        base_color: PURPLE.into(),
                        opaque_render_method: bevy::pbr::OpaqueRendererMethod::Auto,
                        ..Default::default()
                    },
                    extension: crate::player::PlayerViewModelExtension { quantize_steps: 3 },
                };

                let collider = Collider::ball(size);
                (mesh, material, collider, mf(size))
            }
        };

        let friction = Friction::default();
        let restitution = Restitution::default();

        Self {
            item: item.into(),
            rigid_body: RigidBody::Dynamic,
            mass_properties: AdditionalMassProperties::MassProperties(mass),
            collision_groups: LazyLock::force(&ITEM_COLLISION_GROUPS).to_owned(),
            transform: TransformBundle::from_transform(transform),
            visibility: VisibilityBundle::default(),
            mesh: meshes.add(mesh),
            material: materials.add(material),
            collider,
            friction,
            restitution,
        }
    }
}
