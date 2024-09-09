mod equip_item;
mod view_model;
mod world;
use crate::world::GROUND_Y;
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::*,
};
use bevy_rapier3d::prelude::*;
use equip_item::{
    world::{EquipItemWorld, EquipItemWorldBundle, ITEM_COLLISION_GROUPS},
    EquipItem, EquipItemMaterial,
};
use std::sync::LazyLock;
use view_model::{
    single_text_sections, spawn_player_hud, update_player_equipment, LookingAtText,
    PlayerViewModel, PlayerViewModelBundle,
};
use world::{LogicalPlayerEntityBundle, PlayerInWorld, PlayerInWorldBundle};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_plugins(MaterialPlugin::<
                ExtendedMaterial<StandardMaterial, PlayerViewModelExtension>,
            >::default())
            .add_systems(Update, (player_raycast, update_player_equipment));
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

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<EquipItemMaterial>>,
) {
    let player_logical_entity_bundle = LogicalPlayerEntityBundle::default();
    let player_logical_entity = commands.spawn(player_logical_entity_bundle).id();

    let ball_world = EquipItemWorldBundle::from_equip_item(
        EquipItem::Sphere,
        Transform::from_xyz(8.0, 8.0, 8.0),
        &mut meshes,
        &mut materials,
    );

    commands.spawn(ball_world);

    let cube_world = EquipItemWorldBundle::from_equip_item(
        EquipItem::Cube,
        Transform::from_xyz(18.0, 18.0, 18.0),
        &mut meshes,
        &mut materials,
    );

    commands.spawn(cube_world);

    commands.spawn(PlayerInWorldBundle::new(player_logical_entity));
    //
    let vm_bundle = PlayerViewModelBundle::new();
    commands.spawn(vm_bundle);

    spawn_player_hud(&mut commands);
}

fn player_raycast(
    mut commands: Commands,
    player_trans_q: Query<&Transform, With<PlayerInWorld>>,
    mut player_vm_q: Query<(Entity, &mut PlayerViewModel), With<PlayerViewModel>>,
    item_q: Query<(Entity, &EquipItemWorld), With<EquipItemWorld>>,
    mut text_q: Query<&mut Text, With<LookingAtText>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    rapier_context: Res<RapierContext>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<EquipItemMaterial>>,
) {
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
    let (player_vm_entity, mut player_vm) = player_vm_q.single_mut();

    if let Some((entity, _toi)) =
        rapier_context.cast_ray(position, *direction, max_toi, false, filter)
    {
        for (item_entity, eq_item) in &item_q {
            if item_entity == entity {
                last_hit = time.elapsed_seconds_f64();

                text.sections = single_text_sections(&format!("item: {eq_item:?}"));

                if keys.just_pressed(KeyCode::KeyZ) {
                    commands.entity(entity).despawn();
                    let item = Into::<EquipItem>::into(*eq_item);
                    player_vm.add_to_equipment(item);
                }
            }
        }
        if time.elapsed_seconds_f64() - last_hit > seconds_to_update_text || last_hit == 0.0 {
            text.sections = single_text_sections("");
        }
    }
}
