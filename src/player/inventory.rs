use super::items::{
    ball::BallBundle,
    cube::CubeBundle,
    equip::{
        Equip, EquipItem, EquipItemMaterial, EquippedItem, ALL_EQUIPPED_ITEM_VARIANTS,
        EQUIP_TRANSFORM,
    },
};
use bevy::prelude::*;
use std::sync::LazyLock;

pub struct PlayerInventoryPlugin;

impl Plugin for PlayerInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, manage_inventory);
    }
}

#[derive(Debug, Component)]
pub struct PlayerInventory {
    name: Name,
    current_item: EquippedItem,
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self {
            name: Name::new("Inventory"),
            current_item: EquippedItem::default(),
        }
    }
}

impl PlayerInventory {
    pub fn next_item(&mut self) {
        let all_const = ALL_EQUIPPED_ITEM_VARIANTS;
        let all = LazyLock::force(&all_const);
        let idx = all
            .iter()
            .position(|v| v == &self.current_item)
            .expect("no position")
            .to_owned();
        self.current_item = match all.iter().nth(idx + 1) {
            Some(v) => v.clone(),
            None => all.first().expect("should not be empty").clone(),
        }
    }

    pub fn prev_item(&mut self) {
        let all_const = ALL_EQUIPPED_ITEM_VARIANTS;
        let all = LazyLock::force(&all_const);
        let idx = all
            .iter()
            .position(|v| v == &self.current_item)
            .expect("no position")
            .to_owned();
        self.current_item = {
            let last = all.last().expect("should not be empty").clone();
            if idx > 0 {
                all.iter().nth(idx - 1).cloned().unwrap_or(last)
            } else {
                last
            }
        }
    }
}

pub fn setup_inventory(
    parent: &mut ChildBuilder<'_>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<EquipItemMaterial>>,
) {
    let inventory = PlayerInventory::default();

    let mut ball = BallBundle::new(&mut meshes, &mut materials);
    let mut cube = CubeBundle::new(&mut meshes, &mut materials);

    match inventory.current_item {
        EquippedItem::Cube => cube.equip.equip(),
        EquippedItem::Ball => ball.equip.equip(),
        _ => (),
    }

    parent.spawn(ball);
    parent.spawn(cube);
    parent.spawn(inventory);
}

pub(super) const USE_KEY: KeyCode = KeyCode::KeyE;
pub(super) const CYCLE_ITEM_NEXT: KeyCode = KeyCode::KeyT;
pub(super) const CYCLE_ITEM_PREV: KeyCode = KeyCode::KeyG;

fn manage_inventory(
    mut equip_q: Query<(&mut Transform, &mut Equip), With<Equip>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut inventory_q: Query<&mut PlayerInventory>,
) {
    if keys.just_pressed(USE_KEY) {}

    let mut inventory = inventory_q.single_mut();

    if keys.just_pressed(CYCLE_ITEM_NEXT) {
        inventory.next_item();
    }

    if keys.just_pressed(CYCLE_ITEM_PREV) {
        inventory.prev_item();
    }

    let item_that_should_be_vis = &inventory.current_item;
    let e = EQUIP_TRANSFORM;
    let equip_transform = LazyLock::force(&e);

    for (mut transform, mut eq) in &mut equip_q {
        let is_vis = transform.translation == equip_transform.translation;

        if eq.matches(&item_that_should_be_vis) {
            if !is_vis {
                *transform = *equip_transform;
            }
            if !eq.is_equipped() {
                eq.equip();
            }
        } else {
            if is_vis {
                transform.translation.y -= 100.;
            }
            if eq.is_equipped() {
                eq.unequip()
            }
        }
    }
}
