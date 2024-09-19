use super::{
    super::super::{player::PlayerEquipItem, ITEM_COLLISION_GROUPS},
    portal_material::PortalMaterial,
    EquipItemMaterial, WorldEquipCube,
};
use bevy::color::palettes::css::{PURPLE, RED, WHITE};
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::prelude::{ExternalImpulse, RigidBody};
use std::sync::LazyLock;

#[derive(Debug, Component)]
pub struct OuterCube {
    receiver: bool,
    transform: Transform,
}

pub fn update_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    // mut portal_materials: ResMut<Assets<PortalMaterial>>,
    out_cube_q: Query<(Entity, &OuterCube), With<OuterCube>>,
    q: Query<(&WorldEquipCube, &GlobalTransform, &Sleeping), With<WorldEquipCube>>,
) {
    let both_cubes_present = q.iter().count() >= 2;

    for (cube, transform, sleeping) in q.iter() {
        warn!("cube: {cube:?}\nsleeping: {sleeping:?}");
        if both_cubes_present {
            match cube {
                WorldEquipCube::Sender => {
                    if sleeping.sleeping {
                        let mut outer_cube_exists = false;
                        for (entity, outer_cube) in out_cube_q.iter() {
                            if outer_cube.transform != transform.compute_transform()
                                && !outer_cube.receiver
                            {
                                commands.entity(entity).despawn();
                                outer_cube_exists = false;
                            } else {
                                outer_cube_exists = true;
                            }
                        }

                        if !outer_cube_exists {
                            let bundle = PbrBundle {
                                mesh: meshes.add(Cuboid::new(12., 12., 12.)),
                                material: standard_materials.add(Color::from(RED)),
                                transform: (*transform).into(),
                                ..Default::default()
                            };

                            commands.spawn((
                                OuterCube {
                                    receiver: false,
                                    transform: transform.compute_transform(),
                                },
                                bundle,
                            ));
                        }
                    }
                }
                WorldEquipCube::Receiver => {
                    if sleeping.sleeping {
                        let mut outer_cube_exists = false;
                        for (entity, outer_cube) in out_cube_q.iter() {
                            if outer_cube.receiver {
                                if outer_cube.transform != transform.compute_transform() {
                                    commands.entity(entity).despawn();
                                }
                                outer_cube_exists = true;
                            } else {
                                outer_cube_exists = false;
                            }
                        }

                        if !outer_cube_exists {
                            let bundle = PbrBundle {
                                mesh: meshes.add(Cuboid::new(12., 12., 12.)),
                                material: standard_materials.add(Color::from(WHITE)),
                                transform: (*transform).into(),
                                ..Default::default()
                            };

                            commands.spawn((
                                OuterCube {
                                    receiver: true,
                                    transform: transform.compute_transform(),
                                },
                                // this needs to be walls, not a whole collider, otherwise the cube
                                // inside will be inside the collider
                                // Collider::cuboid(6., 6., 6.),
                                bundle,
                            ));
                        }
                    }
                }
            }
        } else {
            for (entity, _outer_cube) in out_cube_q.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}
