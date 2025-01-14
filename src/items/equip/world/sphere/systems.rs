use super::{
    EffectTimer, PlayerEquipSphere, WorldEquipSphere, WorldSphereBundle, MS_TO_CLEAR_EXPLOSION,
};
use crate::items::equip::inventory::Inventory;
use bevy::{color::palettes::css::RED, prelude::*};
use bevy_rapier3d::{
    na::{distance_squared, ComplexField, Point3},
    prelude::{ExternalImpulse, RigidBody},
};
use std::time::Duration;

// ### SYSTEMS

pub fn sphere_dropped(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut inventory_q: Query<&mut Inventory, With<Inventory>>,
    time: Res<Time>,
    rigid_bodies: Query<(Entity, &GlobalTransform), With<RigidBody>>,
    mut q: Query<(Entity, &mut WorldEquipSphere, &GlobalTransform), With<WorldEquipSphere>>,
) {
    let mut inventory = inventory_q.single_mut();
    let mut explosion_bundle = Option::<PbrBundle>::None;
    let base_impulse_magnitude = 100.0;
    let max_distance = 100.;

    for (sphere_entity, mut sphere, transform) in q.iter_mut() {
        if let Some(ref mut timer) = sphere.timer.as_mut() {
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
                    if rb_entity != sphere_entity {
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
                        EffectTimer::from(Timer::new(
                            Duration::from_millis(MS_TO_CLEAR_EXPLOSION),
                            TimerMode::Once,
                        )),
                    ));
                }

                commands.entity(sphere_entity).despawn();
                inventory.add_to_equipment(PlayerEquipSphere::default());
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
