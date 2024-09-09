use super::{EquipItem, EquipItemMaterial};
use bevy::{color::palettes::css::PURPLE, pbr::ExtendedMaterial, prelude::*};
use bevy_rapier3d::prelude::*;
use std::sync::LazyLock;

pub const ITEM_COLLISION_GROUPS: LazyLock<CollisionGroups> = LazyLock::new(|| {
    CollisionGroups::new(
        Group::GROUP_3,
        Group::GROUP_2 | Group::GROUP_1 | Group::GROUP_3,
    )
});

#[derive(Component, Debug, Copy, Clone)]
pub enum EquipItemWorld {
    Sphere,
    Cube,
}

impl From<EquipItem> for EquipItemWorld {
    fn from(value: EquipItem) -> Self {
        match value {
            EquipItem::Cube => Self::Cube,
            EquipItem::Sphere => Self::Sphere,
        }
    }
}

impl Into<EquipItem> for EquipItemWorld {
    fn into(self) -> EquipItem {
        match self {
            Self::Cube => EquipItem::Cube,
            Self::Sphere => EquipItem::Sphere,
        }
    }
}

#[derive(Bundle)]
pub struct EquipItemWorldBundle {
    item: EquipItemWorld,
    collider: Collider,
    collision_groups: CollisionGroups,
    restitution: Restitution,
    friction: Friction,
    rigid_body: RigidBody,
    transform: TransformBundle,
    visibility: VisibilityBundle,
    mesh: Handle<Mesh>,
    material: Handle<EquipItemMaterial>,
}

impl EquipItemWorldBundle {
    pub fn from_equip_item(
        item: EquipItem,
        transform: Transform,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<EquipItemMaterial>>,
    ) -> Self {
        let size = 0.8;

        let (mesh, material, collider) = match item {
            EquipItem::Cube => {
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
                (mesh, material, collider)
            }
            EquipItem::Sphere => {
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
                (mesh, material, collider)
            }
        };

        let friction = Friction::default();
        let restitution = Restitution::default();

        Self {
            item: item.into(),
            rigid_body: RigidBody::Dynamic,
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
