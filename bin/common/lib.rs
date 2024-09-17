use bevy::{color::palettes::css::RED, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use prototype_slenderish::player::PlayerPlugin;

pub fn test_app(player: bool) -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, test_setup);
    if player {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugins(PlayerPlugin);
    } else {
        app.add_systems(Startup, setup_camera);
    }
    app
}

fn test_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1., 1., 1.)),
        material: materials.add(Color::from(RED)),
        transform: Transform::IDENTITY,
        ..Default::default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

fn setup_camera(mut commands: Commands) {
    // camera
    let looking = Vec3::new(-20., 0., 16.5);
    let camera_transform_for_marching_tiles =
        Transform::from_xyz(-195., 190., 66.5).looking_at(looking, Vec3::Y);

    commands.spawn(Camera3dBundle {
        // transform: camera_transform_for_marching_tiles,
        transform: Transform::from_xyz(-25., 25., -10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
