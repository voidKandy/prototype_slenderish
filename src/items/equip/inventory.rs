use std::{sync::LazyLock, time::Duration};

use bevy::{color::palettes::css::WHITE, prelude::*, utils::HashSet};
use bevy_rapier3d::prelude::*;

use crate::player::{
    view_model::{LookingAtText, PlayerViewModel},
    world::PlayerInWorld,
};

use super::{
    player::{single_text_sections, PlayerEquipItem, PlayerEquipItemBundle},
    world::{
        sphere::{GrenadeTimer, WorldSphereState, MS_TO_EXPLODE},
        WorldEquipItem, WorldEquipItemBundle, ITEM_COLLISION_GROUPS,
    },
    EquipItem, EquipItemEvent, EquipItemMaterial,
};

#[derive(Component, Default)]
pub struct Inventory {
    pub currently_equipped: Option<PlayerEquipItem>,
    pub other_items: HashSet<PlayerEquipItem>,
}

impl Inventory {
    /// Adds an item to equipment, equipping it if something already isnt
    pub fn add_to_equipment(&mut self, item: impl Into<PlayerEquipItem>) {
        match self.currently_equipped {
            Some(_) => {
                let _ = self.other_items.insert(item.into());
            }
            None => self.currently_equipped = Some(item.into()),
        }
    }

    fn cycle_equipment_next(&mut self) {
        if let Some(next) = self.other_items.iter().next() {
            let taken = self
                .other_items
                .take(&next.clone())
                .expect("Should be valid get");

            if let Some(item) = self.currently_equipped.take() {
                let _ = self.other_items.insert(item);
            }
            self.currently_equipped = Some(taken);
        }
    }

    pub fn drop_equipment(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<EquipItemMaterial>>,
        player_transform: &Transform,
    ) {
        if let Some(item) = self.currently_equipped.take() {
            let item = Into::<EquipItem>::into(item).into();
            let mut transform = player_transform.clone();
            transform.translation += *player_transform.forward();
            let mut bundle =
                WorldEquipItemBundle::from_equip_item(item, transform, meshes, materials);

            // Maybhaps this would be better implemented as a custom initialization function for an
            // engaged sphere...
            if let EquipItem::Sphere = item {
                if let Some(mat) = materials.get_mut(&bundle.material) {
                    mat.base.emissive = WHITE.into();
                }

                bundle.item = WorldEquipItem::Sphere(WorldSphereState::Engaged {
                    timer: GrenadeTimer::from(Timer::new(
                        Duration::from_millis(MS_TO_EXPLODE),
                        TimerMode::Once,
                    )),
                });
            }
            commands.spawn(bundle).insert(ExternalImpulse {
                impulse: player_transform.forward() * 2.0,
                torque_impulse: Vec3::ZERO,
            });
            self.cycle_equipment_next();
        }
    }
}

/// #### SYSTEMS ####

pub(super) fn update_player_equipment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<EquipItemMaterial>>,
    mut inventory_q: Query<&mut Inventory, With<Inventory>>,
    mut player_vm_q: Query<Entity, With<PlayerViewModel>>,

    mut ev_equip_item: EventWriter<EquipItemEvent>,
    player_trans_q: Query<&Transform, With<PlayerInWorld>>,
    equip_item_q: Query<(Entity, &PlayerEquipItem), With<PlayerEquipItem>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let mut inventory = inventory_q.single_mut();
    let player_vm_entity = player_vm_q.single_mut();

    if keys.just_pressed(KeyCode::KeyN) {
        inventory.cycle_equipment_next();
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        if let Some(i) = inventory.currently_equipped {
            let mut transform = player_trans_q.single().clone();
            // transform.translation += transform.forward() * 2.0;
            inventory.drop_equipment(&mut commands, &mut meshes, &mut materials, &transform);
            ev_equip_item.send(EquipItemEvent::Dropped(i.into()));
        }
    }

    match inventory.currently_equipped {
        Some(item) => {
            for (equip_item_entity, equip_item) in &equip_item_q {
                if *equip_item != item {
                    commands.entity(equip_item_entity).despawn();
                }
            }

            let player_item =
                PlayerEquipItemBundle::from_equip_item(item.into(), &mut meshes, &mut materials);
            commands.entity(player_vm_entity).with_children(|p| {
                p.spawn(player_item);
            });
        }
        None => {
            for (equip_item_entity, _) in &equip_item_q {
                commands.entity(equip_item_entity).despawn();
            }
        }
    }
}

pub(super) fn player_raycast(
    mut commands: Commands,
    mut ev_equip_item: EventWriter<EquipItemEvent>,
    player_trans_q: Query<&Transform, With<PlayerInWorld>>,
    mut inventory_q: Query<&mut Inventory, With<Inventory>>,
    item_q: Query<(Entity, &WorldEquipItem), With<WorldEquipItem>>,
    mut text_q: Query<&mut Text, With<LookingAtText>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
) {
    let mut inventory = inventory_q.single_mut();
    let trans = player_trans_q.single();
    let direction = trans.forward();
    let position = trans.translation + (trans.forward() * 3.0);

    let max_toi = bevy_rapier3d::math::Real::MAX;
    let filter = QueryFilter {
        groups: Some(LazyLock::force(&ITEM_COLLISION_GROUPS).to_owned()),
        ..Default::default()
    };

    let mut text = text_q.single_mut();
    let mut last_hit = 0.0;
    let seconds_to_update_text = 5.0;

    if let Some((entity, _toi)) =
        rapier_context.cast_ray(position, *direction, max_toi, false, filter)
    {
        for (item_entity, eq_item) in &item_q {
            if item_entity == entity {
                last_hit = time.elapsed_seconds_f64();

                text.sections = single_text_sections(&format!("item: {eq_item:?}"));

                if keys.just_pressed(KeyCode::KeyZ) {
                    commands.entity(entity).despawn();
                    let item = Into::<EquipItem>::into(eq_item.to_owned());
                    inventory.add_to_equipment(item);
                    ev_equip_item.send(EquipItemEvent::PickedUp(item));
                }
            }
        }
        if time.elapsed_seconds_f64() - last_hit > seconds_to_update_text || last_hit == 0.0 {
            text.sections = single_text_sections("");
        }
    }
}
