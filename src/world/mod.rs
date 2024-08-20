mod generation;
use bevy::{
    color::palettes::css::{RED, WHITE, YELLOW},
    prelude::{Cuboid, *},
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use generation::{spawn_terrain, spawn_terrain_entities};
use noise::NoiseFn;
use noise::{Fbm, Perlin};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_terrain,
                spawn_terrain_entities,
                spawn_objects,
                spawn_light,
            )
                .chain(),
        );
    }
}

const GROUND_Y: f32 = 0.0;

fn spawn_light(mut commands: Commands) {
    let light = PointLightBundle {
        point_light: PointLight {
            intensity: 20000.0,
            shadows_enabled: true,
            color: YELLOW.into(),
            radius: 600.0,
            range: 600.,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 10.0, 0.0),
        ..Default::default()
    };
    commands.spawn((light, Name::new("main light")));
}

fn spawn_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let blue_cube = PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid::new(0.5, 0.5, 0.5))),
        material: materials.add(Color::srgb(0., 0., 0.5)),
        transform: Transform::from_xyz(0., -1.5, 0.),
        ..Default::default()
    };

    let red_cube = PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid::new(8.0, 8.0, 8.0))),
        material: materials.add(Color::srgb(0.5, 0., 0.)),
        transform: Transform::from_xyz(-4., GROUND_Y, 8.),
        ..Default::default()
    };

    commands.spawn((Name::new("red cube"), red_cube));
    commands.spawn((Name::new("blue cube"), blue_cube));
}
