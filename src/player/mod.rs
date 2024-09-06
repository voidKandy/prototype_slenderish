mod inventory;
mod items;
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::{render_resource::*, view::RenderLayers},
};
use bevy_rapier3d::prelude::*;
use bevy_tnua::math::Vector3;
use std::f32::consts::TAU;

use crate::{
    controller::{CameraConfig, FpsController, FpsControllerInput, LogicalPlayer, RenderPlayer},
    world::GROUND_Y,
    VIEW_MODEL_RENDER_LAYER,
};
use inventory::{setup_inventory, PlayerInventory, PlayerInventoryPlugin};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_player,
                // spawn_shadered_object
            ),
        )
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, PlayerViewModelExtension>,
        >::default())
        .add_plugins(PlayerInventoryPlugin)
        .add_systems(Update, print_player_ray);
    }
}

#[derive(Component)]
pub struct PlayerInWorld;

#[derive(Bundle)]
pub struct PlayerWorldBundle {
    player: PlayerInWorld,
    name: Name,
    render_player: RenderPlayer,
    camera: Camera3dBundle,
}

#[derive(Bundle)]
pub struct PlayerViewModelBundle {
    // inventory: PlayerInventory,
    name: Name,
    camera: Camera3dBundle,
    render_layers: RenderLayers,
}

const SPAWN_POINT: Vec3 = Vec3::new(0.0, GROUND_Y + 5., 0.0);

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct PlayerViewModelExtension {
    // We need to ensure that the bindings of the base material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots 0-99 for the base material.
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

impl PlayerWorldBundle {
    fn new(logical_entity: Entity) -> Self {
        let player = PlayerInWorld;
        let render_player = RenderPlayer { logical_entity };
        let name = Name::new("World Camera");
        let camera = Camera3dBundle {
            projection: Projection::Perspective(PerspectiveProjection {
                fov: TAU / 5.0,
                ..default()
            }),
            // exposure: Exposure::SUNLIGHT,
            ..default()
        };

        Self {
            name,
            render_player,
            player,
            camera,
        }
    }
}

impl PlayerViewModelBundle {
    fn new() -> Self {
        let name = Name::new("View Model Camera");
        let render_layers = RenderLayers::layer(VIEW_MODEL_RENDER_LAYER);
        let camera = Camera3dBundle {
            camera: Camera {
                // Bump the order to render on top of the world model.
                order: 1,
                ..default()
            },
            projection: PerspectiveProjection {
                fov: 70.0_f32.to_radians(),
                ..default()
            }
            .into(),
            ..default()
        };

        Self {
            // inventory: PlayerInventory::default(),
            name,
            camera,
            render_layers,
        }
    }
}

const PLAYER_HEIGHT: f32 = 3.0;

// Note that we have two entities for the player
// One is a "logical" player that handles the physics computation and collision
// The other is a "render" player that is what is displayed to the user
// This distinction is useful for later on if you want to add multiplayer,
// where often time these two ideas are not exactly synced up
fn spawn_logical_player_entity(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Collider::cylinder(PLAYER_HEIGHT / 2.0, 0.5),
            // A capsule can be used but is NOT recommended
            // If you use it, you have to make sure each segment point is
            // equidistant from the translation of the player transform
            // Collider::capsule_y(height / 2.0, 0.5),
            Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            ActiveEvents::COLLISION_EVENTS,
            Velocity::zero(),
            RigidBody::Dynamic,
            Sleeping::disabled(),
            LockedAxes::ROTATION_LOCKED,
            AdditionalMassProperties::Mass(1.0),
            GravityScale(0.0),
            Ccd { enabled: true }, // Prevent clipping when going fast
            TransformBundle::from_transform(Transform::from_translation(SPAWN_POINT)),
            LogicalPlayer,
            FpsControllerInput {
                pitch: -TAU / 12.0,
                yaw: TAU * 5.0 / 8.0,
                ..default()
            },
            FpsController {
                air_acceleration: 80.0,
                ..default()
            },
        ))
        .insert(CameraConfig {
            height_offset: -0.5,
        })
        .id()
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, PlayerViewModelExtension>>>,
    // mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, PlayerViewModelExtension>>>,
) {
    // let flashlight = (
    //     SpotLightBundle {
    //         spot_light: SpotLight {
    //             color: Color::rgba(1.0, 1.0, 0.47, 1.0),
    //             range: 10.0,
    //             intensity: 4000.0 * 1000.0,
    //             outer_angle: 0.5,
    //             inner_angle: 0.4,
    //             shadows_enabled: true,
    //             ..default()
    //         },
    //         // transform: Transform::from_xyz(0.0, 0.25, -0.3),
    //         ..default()
    //     },
    //     Name::new("Flashlight"),
    // );

    let player_logical_entity = spawn_logical_player_entity(&mut commands);

    let mut arm_transform = Transform::from_xyz(0.18, -0.075, -0.25);
    arm_transform.rotate(Quat::from_xyzw(0.1, 0.2, -0.1, 0.));

    commands
        .spawn(PlayerWorldBundle::new(player_logical_entity))
        .with_children(|parent| {
            let vm_bundle = PlayerViewModelBundle::new();
            parent.spawn(vm_bundle);
            setup_inventory(parent, meshes, materials);
        });
}

fn get_view_direction(pitch: f32, yaw: f32) -> Vector3 {
    // Convert angles from degrees to radians if needed (assuming pitch and yaw are in radians)
    // let pitch_rad = pitch.to_radians();
    // let yaw_rad = yaw.to_radians();

    // warn!("PLAYER PITCH: {:?}\nYAW: {:?}", pitch, yaw);
    // Compute the direction vector
    let x = pitch.sin() * yaw.cos();
    let y = pitch.cos();
    let z = pitch.sin() * yaw.sin();

    // Create and return the direction vector
    Vector3::new(x, y, z).normalize() // Normalize the vector to ensure it has unit length
}

fn print_player_ray(
    player_trans_q: Query<&Transform, With<PlayerInWorld>>,
    look_q: Query<&FpsController>,
    rapier_context: Res<RapierContext>,
) {
    let trans = player_trans_q.single();
    // warn!("PLAYER TRANSFORM: {trans:?}");
    let look = look_q.single();
    let direction = get_view_direction(look.pitch, look.yaw);
    let position = trans.translation;
    // warn!("DIRECTION: {direction:?}");

    let max_toi = 4.0;
    let solid = true;
    let filter = QueryFilter::default();

    if let Some((entity, toi)) =
        rapier_context.cast_ray(position, direction, max_toi, solid, filter)
    {
        // The first collider hit has the entity `entity` and it hit after
        // the ray travelled a distance equal to `ray_dir * toi`.
        let hit_point = position + direction * toi;
        // println!("Entity {:?} hit at point {}", entity, hit_point);
    }
}
