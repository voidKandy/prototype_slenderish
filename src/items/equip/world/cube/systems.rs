use super::{
    super::super::{player::PlayerEquipItem, ITEM_COLLISION_GROUPS},
    EquipItemMaterial,
};
use bevy::color::palettes::css::PURPLE;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::prelude::{ExternalImpulse, RigidBody};
use std::sync::LazyLock;
