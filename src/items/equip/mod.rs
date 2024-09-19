use std::{fmt::Debug, sync::LazyLock};
mod events;

use crate::player::PlayerViewModelExtension;
use bevy::{
    pbr::{ExtendedMaterial, StandardMaterial},
    prelude::*,
};
use bevy_rapier3d::prelude::{CollisionGroups, Group};
use inventory::{player_raycast, update_player_equipment, Inventory, WorldEquipHandle};
use player::PlayerEquipItem;
use world::{
    cube::WorldCubeBundle,
    sphere::{
        systems::{sphere_dropped, tick_effect},
        WorldSphereBundle,
    },
};
pub(super) mod inventory;
pub(super) mod player;
pub(super) mod world;

pub(super) struct EquipItemPlugin;

impl Plugin for EquipItemPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_event::<EquipItemEvent>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    update_player_equipment,
                    player_raycast,
                    // handle_equip_item_event,
                    sphere_dropped,
                    tick_effect,
                ),
            );
    }
}

pub const ITEM_COLLISION_GROUPS: LazyLock<CollisionGroups> = LazyLock::new(|| {
    CollisionGroups::new(
        Group::GROUP_3,
        Group::GROUP_2 | Group::GROUP_1 | Group::GROUP_3,
    )
});

pub trait WorldEquipItemBundle<W, P>: Bundle + Debug
where
    Self: Sized,
    W: Debug + Component + Clone,
    P: Debug + Component + Clone + Into<PlayerEquipItem>,
{
    fn world_equip_handle(&self) -> &WorldEquipHandle;
    fn world_to_player(world: &W) -> P;
    fn drop_into_world(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<EquipItemMaterial>>,
        player_transform: &Transform,
    );
    fn bundle(
        transform: Transform,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<EquipItemMaterial>>,
    ) -> Self;
    fn initial_spawn(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<EquipItemMaterial>>,
        transform: &Transform,
    ) {
        let bundle = Self::bundle(*transform, meshes, materials);
        commands.spawn(bundle);
    }
}

pub type EquipItemMaterial = ExtendedMaterial<StandardMaterial, PlayerViewModelExtension>;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<EquipItemMaterial>>,
    // mut ev_equip_item: EventWriter<EquipItemEvent>,
) {
    let t = Transform::from_xyz(8.0, 8.0, 8.0);
    let ball_world = WorldSphereBundle::bundle(t, &mut meshes, &mut materials);

    commands.spawn(ball_world);

    let t = Transform::from_xyz(18.0, 18.0, 18.0);
    let cube_world = WorldCubeBundle::bundle(t, &mut meshes, &mut materials);
    commands.spawn(cube_world);
    let t = Transform::from_xyz(28.0, 18.0, 18.0);
    let cube_world = WorldCubeBundle::bundle(t, &mut meshes, &mut materials);

    commands.spawn(cube_world);

    commands.spawn(Inventory::default());
}
