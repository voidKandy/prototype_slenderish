use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use prototype_slenderish::{
    controller::FpsControllerPlugin, equip_item::EquipItemPlugin, npc::NpcPlugin,
    player::PlayerPlugin, world::WorldPlugin,
};

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
            NpcPlugin,
            // PluginEquipItem,
            // CameraPlugin,
            WorldInspectorPlugin::new(),
        ))
        .run();
}
