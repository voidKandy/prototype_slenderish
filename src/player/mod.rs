mod controller;
pub mod view_model;
pub mod world;
use crate::world::GROUND_Y;
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::*,
};
use bevy_rapier3d::prelude::*;
use controller::FpsControllerPlugin;
use std::sync::LazyLock;
use view_model::{spawn_player_hud, PlayerViewModelBundle};
use world::{LogicalPlayerEntityBundle, PlayerInWorldBundle};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FpsControllerPlugin)
            .add_systems(Startup, spawn_player)
            .add_plugins(MaterialPlugin::<
                ExtendedMaterial<StandardMaterial, PlayerViewModelExtension>,
            >::default());
    }
}

const PLAYER_HEIGHT: f32 = 3.0;
const SPAWN_POINT: Vec3 = Vec3::new(0.0, GROUND_Y + 5., 0.0);
const PLAYER_COLLISION_GROUPS: LazyLock<CollisionGroups> =
    LazyLock::new(|| CollisionGroups::new(Group::GROUP_2, Group::GROUP_1 | Group::GROUP_3));

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct PlayerViewModelExtension {
    #[uniform(100)]
    pub quantize_steps: u32,
}

impl MaterialExtension for PlayerViewModelExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/extended_material.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/extended_material.wgsl".into()
    }
}

fn spawn_player(mut commands: Commands) {
    let player_logical_entity_bundle = LogicalPlayerEntityBundle::default();
    let player_logical_entity = commands.spawn(player_logical_entity_bundle).id();

    commands.spawn(PlayerInWorldBundle::new(player_logical_entity));
    //
    let vm_bundle = PlayerViewModelBundle::new();
    commands.spawn(vm_bundle);

    spawn_player_hud(&mut commands);
}
