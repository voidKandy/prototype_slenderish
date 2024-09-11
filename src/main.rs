mod controller;
mod equip_item;
mod npc;
mod player;
mod rtin;
mod world;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use controller::FpsControllerPlugin;
use equip_item::EquipItemPlugin;
// use items::PluginEquipItem;
use player::PlayerPlugin;
use world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(FpsControllerPlugin)
        // .add_plugins(DefaultPickingPlugins)
        .add_plugins((
            WorldPlugin,
            EquipItemPlugin,
            PlayerPlugin,
            npc::NpcPlugin,
            // PluginEquipItem,
            // CameraPlugin,
            WorldInspectorPlugin::new(),
        ))
        .run();
}

pub const DEFAULT_RENDER_LAYER: usize = 0;
