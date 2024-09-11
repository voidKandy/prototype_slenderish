use bevy::{color::palettes::css::WHITE, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::world::GROUND_Y;

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

#[derive(Component)]
struct Npc;

#[derive(Bundle)]
struct NpcBundle {
    name: Name,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: TransformBundle,
    visibility: VisibilityBundle,
    npc: Npc,
}

impl NpcBundle {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Self {
        let mesh = Cone::default();
        let material = StandardMaterial {
            base_color: WHITE.into(),
            ..Default::default()
        };
        Self {
            name: Name::new("NPC"),
            mesh: meshes.add(mesh),
            material: materials.add(material),
            npc: Npc,
            transform: TransformBundle::from_transform(Transform::from_xyz(
                20.,
                GROUND_Y + 5.,
                20.,
            )),
            visibility: VisibilityBundle::default(),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let bundle = NpcBundle::new(&mut meshes, &mut materials);
    commands.spawn(bundle);
}
