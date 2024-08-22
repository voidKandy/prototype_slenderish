mod controller;
mod player;
mod world;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use controller::FpsControllerPlugin;
use player::PlayerPlugin;
use world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(FpsControllerPlugin)
        .add_plugins((
            WorldPlugin,
            PlayerPlugin,
            // CameraPlugin,
            WorldInspectorPlugin::new(),
        ))
        .add_systems(Startup, spawn_physics)
        // .add_systems(Update, print_ball_altitude)
        .run();
}

/// Used implicitly by all entities without a `RenderLayers` component.
/// Our world model camera and all objects other than the player are on this layer.
/// The light source belongs to both layers.
pub const DEFAULT_RENDER_LAYER: usize = 0;

/// Used by the view model camera and the player's arm.
/// The light source belongs to both layers.
pub const VIEW_MODEL_RENDER_LAYER: usize = 1;

fn spawn_physics(mut commands: Commands) {
    let (x, z) = (1.0, 1.0);

    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.7))
        .insert(TransformBundle::from(Transform::from_xyz(x, 4.0, z)));
}