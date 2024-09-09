mod controller;
mod player;
mod rtin;
mod world;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use controller::FpsControllerPlugin;
// use items::EquipItemPlugin;
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
            PlayerPlugin,
            // EquipItemPlugin,
            // CameraPlugin,
            WorldInspectorPlugin::new(),
        ))
        .run();
}

pub const DEFAULT_RENDER_LAYER: usize = 0;
