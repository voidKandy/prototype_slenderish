use super::{
    world::{cube::PlayerEquipCube, sphere::PlayerEquipSphere},
    EquipItemMaterial,
};
use crate::player::view_model::VIEW_MODEL_RENDER_LAYER;
use bevy::{
    color::palettes::css::PURPLE, pbr::ExtendedMaterial, prelude::*, render::view::RenderLayers,
};
use std::sync::LazyLock;

pub const EQUIP_TRANSFORM: LazyLock<Transform> = LazyLock::new(|| {
    let mut equip_transform = Transform::from_xyz(0.18, -0.075, -0.25);
    equip_transform.rotate(Quat::from_xyzw(0.1, 0.2, -0.1, 0.));
    equip_transform
});

#[derive(Component, PartialEq, Eq, Debug, Clone)]
pub enum PlayerEquipItem {
    Sphere(PlayerEquipSphere),
    Cube(PlayerEquipCube),
}

#[derive(Bundle)]
pub struct PlayerEquipItemBundle {
    item: PlayerEquipItem,
    transform: TransformBundle,
    visibility: VisibilityBundle,
    render_layers: RenderLayers,
    mesh: Handle<Mesh>,
    material: Handle<EquipItemMaterial>,
}

pub fn single_text_sections(str: &str) -> Vec<TextSection> {
    vec![TextSection {
        value: str.to_owned(),
        style: TextStyle {
            font_size: 12.0,
            ..Default::default()
        },
    }]
}

impl PlayerEquipItemBundle {
    pub fn from_player_equip_item(
        item: PlayerEquipItem,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<EquipItemMaterial>>,
    ) -> Self {
        let t = EQUIP_TRANSFORM;
        let transform = LazyLock::force(&t);
        let size = 0.1;

        let (mesh, material) = match &item {
            PlayerEquipItem::Sphere(sphere) => {
                let mesh: Mesh = Sphere::new(size).into();
                let material = ExtendedMaterial {
                    base: StandardMaterial {
                        base_color: PURPLE.into(),
                        opaque_render_method: bevy::pbr::OpaqueRendererMethod::Auto,
                        ..Default::default()
                    },
                    extension: crate::player::PlayerViewModelExtension { quantize_steps: 3 },
                };
                (mesh, material)
            }
            PlayerEquipItem::Cube(ref cube) => {
                let mut cube0: Mesh = Cuboid::new(size, size, size).into();
                if cube.amount_spawned() > &1u8 {
                    let mut cube1: Mesh = Cuboid::new(size, size, size).into();
                    let rotation_angle = 5.0_f32.to_radians();
                    let rotation = Quat::from_rotation_y(rotation_angle);
                    cube1.rotate_by(rotation);
                    let translation = Vec3::new(size * 1.5, 0., 0.);
                    cube1.translate_by(translation);
                    cube0.merge(&cube1);
                }

                let material = ExtendedMaterial {
                    base: StandardMaterial {
                        base_color: PURPLE.into(),
                        opaque_render_method: bevy::pbr::OpaqueRendererMethod::Auto,
                        ..Default::default()
                    },
                    extension: crate::player::PlayerViewModelExtension { quantize_steps: 3 },
                };
                (cube0, material)
            }
        };

        Self {
            item: item.into(),
            transform: TransformBundle::from_transform(transform.to_owned()),
            visibility: VisibilityBundle::default(),
            render_layers: RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
            mesh: meshes.add(mesh),
            material: materials.add(material),
        }
    }
}
