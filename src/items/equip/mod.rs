use crate::player::PlayerViewModelExtension;
use bevy::{
    pbr::{ExtendedMaterial, StandardMaterial},
    prelude::*,
};
use inventory::{player_raycast, update_player_equipment, Inventory};
use world::WorldEquipItemBundle;
mod inventory;
mod player;
mod world;

pub(super) struct EquipItemPlugin;

impl Plugin for EquipItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EquipItemEvent>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    update_player_equipment,
                    player_raycast,
                    update_equip_item_state,
                    world::sphere::tick_sphere,
                    world::sphere::tick_effect,
                ),
            );
    }
}

#[derive(bevy::prelude::Component, Debug, Copy, Clone, PartialEq, Eq)]
pub enum EquipItem {
    Sphere,
    Cube,
}

#[derive(Event, Debug)]
pub enum EquipItemEvent {
    Spawned(EquipItem),
    PickedUp(EquipItem),
    Dropped(EquipItem),
}

pub type EquipItemMaterial = ExtendedMaterial<StandardMaterial, PlayerViewModelExtension>;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<EquipItemMaterial>>,
    mut ev_equip_item: EventWriter<EquipItemEvent>,
) {
    let ball_world = WorldEquipItemBundle::from_equip_item(
        EquipItem::Sphere,
        Transform::from_xyz(8.0, 8.0, 8.0),
        &mut meshes,
        &mut materials,
    );

    commands.spawn(ball_world);
    ev_equip_item.send(EquipItemEvent::Spawned(EquipItem::Sphere));

    let cube_world = WorldEquipItemBundle::from_equip_item(
        EquipItem::Cube,
        Transform::from_xyz(18.0, 18.0, 18.0),
        &mut meshes,
        &mut materials,
    );

    commands.spawn(cube_world);
    ev_equip_item.send(EquipItemEvent::Spawned(EquipItem::Cube));

    commands.spawn(Inventory::default());
}

fn update_equip_item_state(mut ev_equip_item: EventReader<EquipItemEvent>) {
    for ev in ev_equip_item.read() {
        warn!("equip item event: {ev:?}");
        match ev {
            EquipItemEvent::Dropped(item) => {
                warn!("Dropped item: {item:?}");
            }
            EquipItemEvent::PickedUp(item) => {
                warn!("PickedUp item: {item:?}");
            }
            EquipItemEvent::Spawned(item) => {
                warn!("Spawned item: {item:?}");
            }
        }
    }
}
