use bevy::{color::palettes::css::PURPLE, pbr::ExtendedMaterial, prelude::*};
use bevy_rapier3d::prelude::*;
use std::{default, sync::LazyLock};

#[derive(Debug)]
pub(super) struct PickupItemPlugin;

impl Plugin for PickupItemPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Debug, Component)]
pub struct PickupItem;

#[derive(Bundle)]
pub struct PickupItemBundle {
    pub(super) item: PickupItem,
    pub(super) mesh: Handle<Mesh>,
    pub(super) material: Handle<StandardMaterial>,
    transform: TransformBundle,
    visibility: VisibilityBundle,
}

impl PickupItemBundle {
    pub fn page(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        transform: Transform,
    ) -> Self {
        let mesh = Cuboid::new(8.5, 11.0, 0.1);
        let material = Color::WHITE;
        let transform = transform.into();
        Self {
            item: PickupItem,
            mesh: meshes.add(mesh),
            material: materials.add(material),
            transform,
            visibility: VisibilityBundle::default(),
        }
    }
}
