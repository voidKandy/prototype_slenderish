use crate::{player::PlayerViewModelExtension, VIEW_MODEL_RENDER_LAYER};
use bevy::{pbr::ExtendedMaterial, prelude::*, render::view::RenderLayers};
use std::{fmt::Debug, sync::LazyLock};

pub type EquipItemMaterial = ExtendedMaterial<StandardMaterial, PlayerViewModelExtension>;

#[derive(Component)]
pub struct Equip {
    equipped: bool,
    item: EquippedItem,
}

/// Determines cycling logic
pub const ALL_EQUIPPED_ITEM_VARIANTS: LazyLock<Vec<EquippedItem>> =
    LazyLock::new(|| vec![EquippedItem::None, EquippedItem::Cube, EquippedItem::Ball]);

#[derive(Debug, Default, Component, PartialEq, Eq, Clone)]
pub enum EquippedItem {
    #[default]
    None,
    Cube,
    Ball,
}

impl Default for Equip {
    fn default() -> Self {
        Self {
            equipped: false,
            item: EquippedItem::default(),
        }
    }
}

impl Equip {
    pub fn ball() -> Self {
        Self {
            item: EquippedItem::Ball,
            ..Default::default()
        }
    }
    pub fn cube() -> Self {
        Self {
            item: EquippedItem::Cube,
            ..Default::default()
        }
    }

    pub fn is_equipped(&self) -> bool {
        self.equipped
    }

    pub fn equip(&mut self) {
        self.equipped = true;
    }
    pub fn unequip(&mut self) {
        self.equipped = false;
    }
    pub fn matches(&self, other: &EquippedItem) -> bool {
        &self.item == other
    }
}

pub const EQUIP_TRANSFORM: LazyLock<Transform> = LazyLock::new(|| {
    let mut equip_transform = Transform::from_xyz(0.18, -0.075, -0.25);
    equip_transform.rotate(Quat::from_xyzw(0.1, 0.2, -0.1, 0.));
    equip_transform
});

pub trait EquipItem: Bundle {
    fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<EquipItemMaterial>>,
    ) -> Self;
    fn mesh() -> Mesh;
    fn material() -> EquipItemMaterial;
}
