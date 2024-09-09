use super::{EquipItem, EquipItemMaterial};
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

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum EquipItemPlayer {
    Sphere,
    Cube,
}

impl From<EquipItem> for EquipItemPlayer {
    fn from(value: EquipItem) -> Self {
        match value {
            EquipItem::Cube => Self::Cube,
            EquipItem::Sphere => Self::Sphere,
        }
    }
}

impl Into<EquipItem> for EquipItemPlayer {
    fn into(self) -> EquipItem {
        match self {
            Self::Cube => EquipItem::Cube,
            Self::Sphere => EquipItem::Sphere,
        }
    }
}

#[derive(Bundle)]
pub struct EquipItemPlayerBundle {
    item: EquipItemPlayer,
    transform: TransformBundle,
    visibility: VisibilityBundle,
    render_layers: RenderLayers,
    mesh: Handle<Mesh>,
    material: Handle<EquipItemMaterial>,
}

impl EquipItemPlayerBundle {
    pub fn from_equip_item(
        item: EquipItem,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<EquipItemMaterial>>,
    ) -> Self {
        let t = EQUIP_TRANSFORM;
        let transform = LazyLock::force(&t);
        let size = 0.1;

        let (mesh, material) = match item {
            EquipItem::Sphere => {
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
            EquipItem::Cube => {
                let mesh: Mesh = Cuboid::new(size, size, size).into();
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
