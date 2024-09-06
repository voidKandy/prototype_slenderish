use std::sync::LazyLock;

use super::equip::{Equip, EquipItem, EquipItemMaterial, EQUIP_TRANSFORM};
use bevy::{
    color::palettes::css::PURPLE,
    pbr::{ExtendedMaterial, OpaqueRendererMethod},
    prelude::*,
};
use noise::{Fbm, NoiseFn, Perlin};

pub(super) struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, rotate_things);
    }
}

#[derive(Bundle)]
pub struct BallBundle {
    pub equip: Equip,
    name: Name,
    // light: DirectionalLightBundle,
    material_mesh: MaterialMeshBundle<EquipItemMaterial>,
}

fn rotate_things(mut q: Query<&mut Transform, With<Equip>>, time: Res<Time>) {
    let noise = Fbm::<Perlin>::new(0);
    //
    let speed = 0.5f64;
    for mut t in &mut q {
        let (x_coord, y_coord) = (time.delta_seconds() as f64 * speed, 0f64);
        //
        let sample = noise.get([x_coord, y_coord]) as f32;
        let z = time.delta_seconds() * (sample * speed as f32 / 2.0);
        if sample > 0.5 {
            t.rotate_y(sample + y_coord as f32);
        } else {
            // r.accumulated_x += sample;
            t.rotate_x(sample + x_coord as f32);
        };
        t.rotate_z(z);
    }
}

impl EquipItem for BallBundle {
    fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<EquipItemMaterial>>,
    ) -> Self {
        let name = Name::new("Ball Bundle");
        let light = DirectionalLightBundle {
            transform: Transform::from_xyz(1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        };
        let material_mesh = MaterialMeshBundle {
            mesh: meshes.add(Self::mesh()),
            material: materials.add(Self::material()),
            transform: LazyLock::force(&EQUIP_TRANSFORM).to_owned(),
            ..Default::default()
        };
        BallBundle {
            equip: Equip::ball(),
            material_mesh,
            name,
            // light,
        }
    }
    fn mesh() -> Mesh {
        Sphere::new(0.08).into()
    }

    fn material(
    ) -> bevy::pbr::ExtendedMaterial<StandardMaterial, crate::player::PlayerViewModelExtension>
    {
        ExtendedMaterial {
            base: StandardMaterial {
                base_color: PURPLE.into(),
                // can be used in forward or deferred mode.
                opaque_render_method: OpaqueRendererMethod::Auto,
                // in deferred mode, only the PbrInput can be modified (uvs, color and other material properties),
                // in forward mode, the output can also be modified after lighting is applied.
                // see the fragment shader `extended_material.wgsl` for more info.
                // Note: to run in deferred mode, you must also add a `DeferredPrepass` component to the camera and either
                // change the above to `OpaqueRendererMethod::Deferred` or add the `DefaultOpaqueRendererMethod` resource.
                ..Default::default()
            },
            extension: crate::player::PlayerViewModelExtension { quantize_steps: 3 },
        }
    }
}
