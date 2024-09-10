use std::time::Duration;

use bevy::{
    color::palettes::css::{RED, WHITE},
    prelude::*,
};
use bevy_rapier3d::{
    na::{distance_squared, ComplexField, OPoint, Point3},
    prelude::{Collider, ExternalForce, ExternalImpulse, RigidBody},
};

use crate::equip_item::{inventory::Inventory, EquipItem, EquipItemEvent, EquipItemMaterial};

use super::WorldEquipItem;

#[derive(Debug, Clone, Default)]
pub enum WorldSphereState {
    Engaged {
        timer: GrenadeTimer,
    },
    #[default]
    Disengaged,
}

#[derive(Component, Debug, Clone)]
pub struct GrenadeTimer(Timer);

#[derive(Component, Debug, Clone)]
pub struct EffectTimer(Timer);

impl From<Timer> for GrenadeTimer {
    fn from(value: Timer) -> Self {
        Self(value)
    }
}

impl From<Timer> for EffectTimer {
    fn from(value: Timer) -> Self {
        Self(value)
    }
}

pub const SECONDS_TO_EXPLODE: u64 = 2;

fn angle_between_points(p1: Point3<f32>, p2: Point3<f32>) -> f32 {
    // Compute the dot product
    let dot_product = p1.x * p2.x + p1.y * p2.y + p1.z * p2.z;
    // Compute the magnitudes
    let mag1 = (p1.x * p1.x + p1.y * p1.y + p1.z * p1.z).sqrt();
    let mag2 = (p2.x * p2.x + p2.y * p2.y + p2.z * p2.z).sqrt();
    // Compute the cosine of the angle
    let cos_theta = dot_product / (mag1 * mag2);
    // Clamp cosine value to the range [-1, 1] to avoid out-of-range errors in arccos
    let cos_theta = cos_theta.max(-1.0).min(1.0);
    // Compute the angle in radians
    let theta = cos_theta.acos();
    // Convert to degrees
    theta.to_degrees()
}

pub fn tick_sphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ev_equip_item: EventWriter<EquipItemEvent>,
    mut inventory_q: Query<&mut Inventory, With<Inventory>>,
    time: Res<Time>,
    rigid_bodies: Query<(Entity, &GlobalTransform), With<RigidBody>>,
    mut q: Query<(Entity, &mut WorldEquipItem, &GlobalTransform), With<WorldEquipItem>>,
) {
    let mut inventory = inventory_q.single_mut();
    let mut explosion_bundle = Option::<PbrBundle>::None;
    let base_impulse_magnitude = 100.0;
    let max_distance = 100.;

    for (world_item_entity, mut item, transform) in q.iter_mut() {
        if let WorldEquipItem::Sphere(ref mut state) = &mut *item {
            if let WorldSphereState::Engaged { timer } = state {
                timer.0.tick(time.delta());

                if timer.0.finished() {
                    let explosion_transform = transform.compute_transform();

                    explosion_bundle = Some(PbrBundle {
                        mesh: meshes.add(Sphere::new(max_distance.sqrt())),
                        material: materials.add(Color::from(RED)),
                        transform: explosion_transform,
                        ..Default::default()
                    });
                    for (rb_entity, t) in &rigid_bodies {
                        if rb_entity != world_item_entity {
                            let explosion_point = Point3::new(
                                explosion_transform.translation.x,
                                explosion_transform.translation.y,
                                explosion_transform.translation.z,
                            );

                            let rb_transform = t.compute_transform();
                            let rb_point = Point3::new(
                                rb_transform.translation.x,
                                rb_transform.translation.y,
                                rb_transform.translation.z,
                            );

                            let distance = distance_squared(&explosion_point, &rb_point);

                            if distance < max_distance {
                                let distance_scale = 1.0 - (distance / max_distance).powi(2);

                                let impulse_magnitude = base_impulse_magnitude * distance_scale;
                                let direction = Vec3::new(
                                    rb_point.x - explosion_point.x,
                                    rb_point.y - explosion_point.y,
                                    rb_point.z - explosion_point.z,
                                )
                                .normalize(); // Normalize to get direction

                                let impulse = direction * impulse_magnitude;

                                // Insert the external impulse component
                                commands.entity(rb_entity).insert(ExternalImpulse {
                                    impulse: Vec3::new(impulse.x, impulse.y, impulse.z),
                                    torque_impulse: Vec3::ZERO, // Example torque, adjust as needed
                                });
                                warn!(
                                    "applying force to {:?}: magnitude = {}, direction = {:?}",
                                    rb_entity, impulse_magnitude, direction
                                );
                            }
                        }
                    }

                    if let Some(bundle) = explosion_bundle.take() {
                        commands.spawn((
                            bundle,
                            EffectTimer::from(Timer::new(Duration::from_secs(1), TimerMode::Once)),
                        ));
                    }

                    commands.entity(world_item_entity).despawn();
                    let item = Into::<EquipItem>::into(item.to_owned());
                    inventory.add_to_equipment(item);

                    ev_equip_item.send(EquipItemEvent::PickedUp(item));
                }
            }
        }
    }
}

pub fn tick_effect(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut EffectTimer), With<EffectTimer>>,
) {
    for (entity, mut timer) in q.iter_mut() {
        timer.0.tick(time.delta());

        if timer.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}
