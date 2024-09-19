pub mod systems;
use crate::items::equip::{
    inventory::WorldEquipHandle, player::PlayerEquipItem, EquipItemMaterial, WorldEquipItemBundle,
    ITEM_COLLISION_GROUPS,
};
use bevy::{
    color::palettes::css::{PURPLE, WHITE},
    pbr::ExtendedMaterial,
    prelude::*,
};
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::prelude::{ExternalImpulse, RigidBody};
use std::{sync::LazyLock, time::Duration};

#[derive(Debug, Clone, Default, Component)]
pub struct WorldEquipSphere {
    timer: Option<GrenadeTimer>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Component)]
pub struct PlayerEquipSphere;

impl Into<PlayerEquipItem> for PlayerEquipSphere {
    fn into(self) -> PlayerEquipItem {
        PlayerEquipItem::Sphere(self)
    }
}

#[derive(Bundle, Debug)]
pub struct WorldSphereBundle {
    pub(super) sphere: WorldEquipSphere,
    pub(super) mesh: Handle<Mesh>,
    pub(super) material: Handle<EquipItemMaterial>,
    handle: WorldEquipHandle,
    collider: Collider,
    collision_groups: CollisionGroups,
    restitution: Restitution,
    friction: Friction,
    rigid_body: RigidBody,
    transform: TransformBundle,
    visibility: VisibilityBundle,
    ccd: Ccd,
}

impl WorldEquipSphere {
    pub fn new_with_timer() -> Self {
        Self {
            timer: Some(GrenadeTimer::from(Timer::new(
                Duration::from_millis(MS_TO_EXPLODE),
                TimerMode::Once,
            ))),
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct GrenadeTimer(Timer);
pub const MS_TO_EXPLODE: u64 = 800;

#[derive(Component, Debug, Clone)]
pub struct EffectTimer(Timer);
pub const MS_TO_CLEAR_EXPLOSION: u64 = 500;

impl From<Timer> for GrenadeTimer {
    fn from(value: Timer) -> Self {
        Self(value)
    }
}

impl From<Timer> for EffectTimer {
    fn from(value: Timer) -> Self {
        Self(value)
    }
}

impl WorldEquipItemBundle<WorldEquipSphere, PlayerEquipSphere> for WorldSphereBundle {
    fn world_equip_handle(&self) -> &WorldEquipHandle {
        &self.handle
    }

    fn drop_into_world(
        player_item: PlayerEquipSphere,
        inventory: &mut crate::items::equip::inventory::Inventory,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<EquipItemMaterial>>,
        player_transform: &Transform,
    ) {
        let mut transform = player_transform.clone();
        transform.translation += *player_transform.forward();
        let mut bundle = Self::bundle(transform, meshes, materials);
        if let Some(mat) = materials.get_mut(&bundle.material) {
            mat.base.emissive = WHITE.into();
        }
        bundle.sphere = WorldEquipSphere::new_with_timer();
        commands.spawn(bundle).insert(ExternalImpulse {
            impulse: player_transform.forward() * 2.0,
            torque_impulse: Vec3::ZERO,
        });
        inventory.cycle_equipment_next();
    }

    fn bundle(
        transform: Transform,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<EquipItemMaterial>>,
    ) -> WorldSphereBundle {
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
        let restitution = Restitution {
            coefficient: 0.2,
            combine_rule: CoefficientCombineRule::Min,
        };
        let ccd = Ccd::enabled();
        let friction = Friction::default();
        let sphere = WorldEquipSphere::default();

        WorldSphereBundle {
            handle: WorldEquipHandle::Sphere,
            sphere,
            rigid_body: RigidBody::Dynamic,
            collision_groups: LazyLock::force(&ITEM_COLLISION_GROUPS).to_owned(),
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
