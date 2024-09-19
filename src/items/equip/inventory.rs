use super::{
    player::{single_text_sections, PlayerEquipItem, PlayerEquipItemBundle},
    world::{
        cube::{PlayerEquipCube, WorldCubeBundle},
        sphere::{PlayerEquipSphere, WorldSphereBundle},
    },
    EquipItemMaterial, WorldEquipItemBundle, ITEM_COLLISION_GROUPS,
};
use crate::player::{
    view_model::{LookingAtText, PlayerViewModel},
    world::PlayerInWorld,
};
use bevy::{
    prelude::*,
    utils::{hashbrown::HashMap, HashSet},
};
use bevy_rapier3d::prelude::*;
use core::panic;
use std::sync::LazyLock;

#[derive(Component, Clone, Copy, Debug)]
pub enum WorldEquipHandle {
    Cube,
    Sphere,
}

impl Into<PlayerEquipItem> for WorldEquipHandle {
    fn into(self) -> PlayerEquipItem {
        match self {
            Self::Cube => PlayerEquipItem::Cube(PlayerEquipCube::default()),
            Self::Sphere => PlayerEquipItem::Sphere(PlayerEquipSphere::default()),
        }
    }
}

#[derive(Component, Default)]
pub struct Inventory {
    pub currently_equipped: Option<PlayerEquipItem>,
    pub other_items: HashMap<u8, PlayerEquipItem>,
}

impl Inventory {
    const CUBE: u8 = 0;
    const SPHERE: u8 = 1;

    fn item_code(item: &PlayerEquipItem) -> u8 {
        match item {
            PlayerEquipItem::Cube(_) => Self::CUBE,
            PlayerEquipItem::Sphere(_) => Self::SPHERE,
        }
    }

    /// Adds an item to equipment, equipping it if something already isnt
    pub fn add_to_equipment(&mut self, item: impl Into<PlayerEquipItem>) {
        let item: PlayerEquipItem = item.into();
        warn!(
            "adding {item:?} to equipment\ncurrently: {:?}",
            self.currently_equipped
        );
        match self.currently_equipped {
            Some(ref mut equipped) => {
                if let (PlayerEquipItem::Cube(_), PlayerEquipItem::Cube(ref mut cube)) =
                    (&item, equipped)
                {
                    if cube.amount_spawned() < &2u8 {
                        warn!("increasing cube count");
                        cube.increase_count();
                    }
                } else if let Some(item) = self.other_items.get_mut(&Self::item_code(&item)) {
                    if let PlayerEquipItem::Cube(cube) = item {
                        if cube.amount_spawned() < &2u8 {
                            warn!("increasing cube count");
                            cube.increase_count();
                        }
                    }
                } else {
                    let _ = self.other_items.insert(Self::item_code(&item), item);
                }
            }
            None => self.currently_equipped = Some(item),
        }
    }

    fn cycle_equipment_next(&mut self) {
        if let Some(next_code) = self
            .other_items
            .iter()
            .next()
            .and_then(|(c, _)| Some(c.to_owned()))
        {
            let taken = self
                .other_items
                .remove(&next_code)
                .expect("Should be valid get");

            if let Some(item) = self.currently_equipped.take() {
                let _ = self.other_items.insert(Self::item_code(&item), item);
            }
            self.currently_equipped = Some(taken);
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

    // mut ev_equip_item: EventWriter<EquipItemEvent>,
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
        if let Some(i) = &inventory.currently_equipped {
            let mut transform = player_trans_q.single().clone();
            // transform.translation += transform.forward() * 2.0;
            // inventory.drop_equipment(&mut commands, &mut meshes, &mut materials, &transform);
            if let Some(mut item) = inventory.currently_equipped.take() {
                match item {
                    PlayerEquipItem::Sphere(_) => {
                        WorldSphereBundle::drop_into_world(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &transform,
                        );
                        inventory.cycle_equipment_next();
                    }
                    PlayerEquipItem::Cube(ref mut cube) => {
                        WorldCubeBundle::drop_into_world(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &transform,
                        );

                        if cube.amount_spawned() > &1u8 {
                            cube.decrease_count();
                            inventory.currently_equipped = Some(item);
                        } else {
                            inventory.cycle_equipment_next();
                        }
                    }
                }
            }
            // ev_equip_item.send(EquipItemEvent::Dropped(i.into()));
        }
    }

    match &inventory.currently_equipped {
        Some(item) => {
            for (equip_item_entity, equip_item) in &equip_item_q {
                if *equip_item != *item {
                    commands.entity(equip_item_entity).despawn();
                } else {
                    // if let PlayerEquipItem::Cube(ref mut cube) = item {
                    //     cube.increase_count();
                    // }
                }
            }
            let player_item = PlayerEquipItemBundle::from_player_equip_item(
                item.to_owned(),
                &mut meshes,
                &mut materials,
            );
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
    // mut ev_equip_item: EventWriter<EquipItemEvent>,
    player_trans_q: Query<&Transform, With<PlayerInWorld>>,
    mut inventory_q: Query<&mut Inventory, With<Inventory>>,
    item_q: Query<(Entity, &WorldEquipHandle), With<WorldEquipHandle>>,
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
        for (item_entity, eq_item_handle) in item_q.into_iter() {
            if item_entity == entity {
                last_hit = time.elapsed_seconds_f64();

                text.sections = single_text_sections(&format!("item: {eq_item_handle:?}",));

                if keys.just_pressed(KeyCode::KeyZ) {
                    commands.entity(entity).despawn();
                    // let item = Into::<EquipItem>::into(eq_item.to_owned());
                    inventory.add_to_equipment(*eq_item_handle);
                    // ev_equip_item.send(EquipItemEvent::PickedUp(item));
                }
            }
        }
        if time.elapsed_seconds_f64() - last_hit > seconds_to_update_text || last_hit == 0.0 {
            text.sections = single_text_sections("");
        }
    }
}
