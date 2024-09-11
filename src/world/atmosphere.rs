use super::icosahedron;
use bevy::{
    pbr::{CascadeShadowConfig, CascadeShadowConfigBuilder, NotShadowCaster},
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(super) struct SkyMaterial {}

impl Material for SkyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/animate_shader.wgsl".into()
    }
}

pub fn setup_atmosphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
) {
    // let sphere = PbrBundle {
    //     mesh: meshes.add(icosahedron(0).into_mesh(true, 5.)),
    //     material: materials.add(StandardMaterial {
    //         base_color: Color::WHITE,
    //         metallic: 1.0,
    //         perceptual_roughness: 0.,
    //         reflectance: 1.,
    //         ..Default::default()
    //     }),
    //     transform: Transform::from_xyz(10., 2., 0.),
    //     ..Default::default()
    // };
    // commands.spawn(sphere);

    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.3,
        maximum_distance: 3.0,
        ..default()
    }
    .build();

    // Sun
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::srgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(100.0, 10.0, -100.0)
            .looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::Y),
        cascade_shadow_config,
        ..default()
    });

    let half_size = Vec2::new(500.0, 500.0);
    let distance = 1000.;

    let mut com = commands.spawn((Name::new("sky"), NotShadowCaster));

    let planes = [
        (Vec3::new(0.0, distance, 0.0), Vec3::Y), // Plane facing -Y
        (Vec3::new(0.0, distance * 100., 0.0), -Vec3::Y), // Plane facing -Y
    ];

    // Loop through the positions and spawn planes
    for (position, normal) in planes.iter() {
        com.with_children(|p| {
            p.spawn(MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(Plane3d {
                    normal: Dir3::new(*normal).unwrap(),
                    half_size,
                })),
                material: materials.add(SkyMaterial {}),
                transform: Transform::from_translation(*position),
                ..default()
            });
        });
    }
}
