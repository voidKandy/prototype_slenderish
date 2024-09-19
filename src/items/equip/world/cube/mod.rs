// pub mod portal_material;
pub mod systems;
use crate::items::equip::inventory::WorldEquipHandle;
use crate::items::equip::player::PlayerEquipItem;
use crate::items::equip::{EquipItemMaterial, WorldEquipItemBundle};

use super::super::ITEM_COLLISION_GROUPS;
use bevy::color::palettes::css::PURPLE;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::prelude::{ExternalImpulse, RigidBody};
use std::sync::LazyLock;

#[derive(Debug, Clone, Default, Component)]
pub enum WorldEquipCube {
    Sender,
    #[default]
    Receiver,
}

#[derive(Debug, Clone, Component, PartialEq, Eq)]
pub struct PlayerEquipCube(u8);

impl Default for PlayerEquipCube {
    fn default() -> Self {
        Self(1)
    }
}

impl PlayerEquipCube {
    pub fn amount_spawned(&self) -> &u8 {
        &self.0
    }

    pub fn increase_count(&mut self) {
        if self.0 < 2 {
            self.0 += 1;
        }
    }

    pub fn decrease_count(&mut self) {
        if self.0 > 0 {
            self.0 -= 1;
        }
    }
}

impl Into<PlayerEquipItem> for PlayerEquipCube {
    fn into(self) -> PlayerEquipItem {
        PlayerEquipItem::Cube(self)
    }
}

#[derive(Bundle, Debug)]
pub struct WorldCubeBundle {
    handle: WorldEquipHandle,
    pub(super) world_cube: WorldEquipCube,
    // player_cube: PlayerEquipCube,
    pub(super) mesh: Handle<Mesh>,
    pub(super) material: Handle<EquipItemMaterial>,
    collider: Collider,
    collision_groups: CollisionGroups,
    restitution: Restitution,
    friction: Friction,
    rigid_body: RigidBody,
    sleeping: Sleeping,
    transform: TransformBundle,
    visibility: VisibilityBundle,
    ccd: Ccd,
}

impl WorldEquipItemBundle<WorldEquipCube, PlayerEquipCube> for WorldCubeBundle {
    fn world_equip_handle(&self) -> &WorldEquipHandle {
        &self.handle
    }

    fn drop_into_world(
        mut player_item: PlayerEquipCube,
        inventory: &mut crate::items::equip::inventory::Inventory,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<EquipItemMaterial>>,
        player_transform: &Transform,
    ) {
        let mut transform = player_transform.clone();
        transform.translation += *player_transform.forward();
        let mut bundle = Self::bundle(transform, meshes, materials);

        bundle.world_cube = match player_item.amount_spawned() {
            1 => WorldEquipCube::Receiver,
            2 => WorldEquipCube::Sender,
            other => panic!("{other} should be an impossible cube count"),
        };

        commands.spawn(bundle).insert(ExternalImpulse {
            impulse: player_transform.forward() * 2.0,
            torque_impulse: Vec3::ZERO,
        });
        if player_item.amount_spawned() > &1u8 {
            player_item.decrease_count();
            inventory.currently_equipped = Some(player_item.into());
        } else {
            inventory.cycle_equipment_next();
        }
    }

    fn bundle(
        transform: Transform,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<EquipItemMaterial>>,
    ) -> Self {
        let size = 0.8;
        let mesh: Mesh = Cuboid::new(size, size, size).into();
        let material = ExtendedMaterial {
            base: StandardMaterial {
                base_color: PURPLE.into(),
                opaque_render_method: bevy::pbr::OpaqueRendererMethod::Auto,
                ..Default::default()
            },
            extension: crate::player::PlayerViewModelExtension { quantize_steps: 3 },
        };

        let collider = Collider::cuboid(size / 2., size / 2., size / 2.);
        let restitution = Restitution {
            coefficient: 0.2,
            combine_rule: CoefficientCombineRule::Min,
        };
        let ccd = Ccd::enabled();
        let friction = Friction::default();
        let world_cube = WorldEquipCube::default();

        WorldCubeBundle {
            world_cube,
            rigid_body: RigidBody::Dynamic,
            sleeping: Sleeping::default(),
            collision_groups: LazyLock::force(&ITEM_COLLISION_GROUPS).to_owned(),
            handle: WorldEquipHandle::Cube,
            ccd,
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
