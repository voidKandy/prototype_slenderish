use super::{PLAYER_COLLISION_GROUPS, PLAYER_HEIGHT, SPAWN_POINT};
use crate::controller::{
    CameraConfig, FpsController, FpsControllerInput, LogicalPlayer, RenderPlayer,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::{f32::consts::TAU, sync::LazyLock};

#[derive(Component)]
pub struct PlayerInWorld;

#[derive(Bundle)]
pub struct PlayerInWorldBundle {
    player: PlayerInWorld,
    name: Name,
    render_player: RenderPlayer,
    camera: Camera3dBundle,
}

#[derive(Bundle)]
pub struct LogicalPlayerEntityBundle {
    collider: Collider,
    friction: Friction,
    restitution: Restitution,
    active_events: ActiveEvents,
    collision_groups: CollisionGroups,
    velocity: Velocity,
    rigid_body: RigidBody,
    sleeping: Sleeping,
    locked_axes: LockedAxes,
    additional_mass_properties: AdditionalMassProperties,
    gravity_scale: GravityScale,
    ccd: Ccd,
    transform: TransformBundle,
    logical_player: LogicalPlayer,
    fps_controller_input: FpsControllerInput,
    fps_controller: FpsController,
    camera_config: CameraConfig,
}

impl PlayerInWorldBundle {
    pub fn new(logical_entity: Entity) -> Self {
        let player = PlayerInWorld;
        let render_player = RenderPlayer { logical_entity };
        let name = Name::new("World Camera");
        let camera = Camera3dBundle {
            projection: Projection::Perspective(PerspectiveProjection {
                fov: TAU / 5.0,
                ..default()
            }),
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

impl Default for LogicalPlayerEntityBundle {
    fn default() -> Self {
        Self {
            collider: Collider::cylinder(PLAYER_HEIGHT / 2.0, 0.5),
            friction: Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            restitution: Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            active_events: ActiveEvents::COLLISION_EVENTS,
            velocity: Velocity::zero(),
            rigid_body: RigidBody::Dynamic,
            sleeping: Sleeping::disabled(),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            additional_mass_properties: AdditionalMassProperties::Mass(1.0),
            gravity_scale: GravityScale(0.0),
            ccd: Ccd { enabled: true }, // Prevent clipping when going fast
            transform: TransformBundle::from_transform(Transform::from_translation(SPAWN_POINT)),
            logical_player: LogicalPlayer,
            fps_controller_input: FpsControllerInput {
                pitch: -TAU / 12.0,
                yaw: TAU * 5.0 / 8.0,
                ..default()
            },
            fps_controller: FpsController {
                air_acceleration: 80.0,
                ..default()
            },
            collision_groups: LazyLock::force(&PLAYER_COLLISION_GROUPS).to_owned(),
            camera_config: CameraConfig {
                height_offset: -0.5,
            },
        }
    }
}
