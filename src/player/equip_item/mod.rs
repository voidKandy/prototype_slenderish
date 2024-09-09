use crate::player::PlayerViewModelExtension;
use bevy::pbr::{ExtendedMaterial, StandardMaterial};
pub mod player;
pub mod world;

#[derive(bevy::prelude::Component, Debug, Copy, Clone)]
pub enum EquipItem {
    Sphere,
    Cube,
}

pub type EquipItemMaterial = ExtendedMaterial<StandardMaterial, PlayerViewModelExtension>;
