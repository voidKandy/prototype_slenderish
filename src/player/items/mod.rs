pub mod ball;
pub mod cube;
pub mod equip;
use bevy::prelude::*;

use self::{ball::BallPlugin, cube::CubePlugin};
pub struct EquipPlugin;

impl Plugin for EquipPlugin {
    fn build(&self, app: &mut App) {
        BallPlugin.build(app);
        CubePlugin.build(app);
    }
}

// fn keyboard_input(
//     mut commands: Commands,
//
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, PlayerViewModelExtension>>>,
//     mut q: Query<&mut EquipItem, With<PlayerModel>>,
//     ball_q: Query<Option<Entity>, With<Ball>>,
//     cube_q: Query<Option<Entity>, With<Cube>>,
//     keys: Res<ButtonInput<KeyCode>>,
// ) {
//     // if keys.just_pressed(USE_KEY) {}
//     let mut is_ball = false;
//     let mut is_cube = false;
//     let mut is_none = false;
//
//     for b in &ball_q {
//         if let Some(e) = b {
//             commands.entity(e).despawn();
//             is_ball = true;
//         }
//     }
//
//     for c in &cube_q {
//         if let Some(e) = c {
//             commands.entity(e).despawn();
//             is_cube = true;
//         }
//     }
//
//     for mut item in q.iter_mut() {
//         warn!("item before: {item:?}");
//         if keys.just_pressed(CYCLE_ITEM_RIGHT) {
//             item.next()
//         }
//         if keys.just_pressed(CYCLE_ITEM_LEFT) {
//             item.prev()
//         }
//         warn!("item after: {item:?}");
//         let mut arm_transform = Transform::from_xyz(0.18, -0.075, -0.25);
//         arm_transform.rotate(Quat::from_xyzw(0.1, 0.2, -0.1, 0.));
//
//         let render_layers = RenderLayers::layer(VIEW_MODEL_RENDER_LAYER);
//         if let Some((mesh, material)) = item.mesh_and_material() {
//             let bundle = MaterialMeshBundle {
//                 mesh: meshes.add(mesh),
//                 transform: arm_transform,
//                 material: materials.add(material),
//                 ..default()
//             };
//             commands
//                 .spawn((bundle, render_layers.clone()))
//                 .with_children(|p| {
//                     if let Some(bundle) = item.additional_bundle() {
//                         bundle.spawn(p, render_layers)
//                     }
//                 });
//         }
//     }
// }
