#[path = "../bin/common/lib.rs"]
mod common;
use bevy::prelude::*;

pub fn main() {
    let mut app = common::test_app(false);
    app.add_systems(Startup, setup_page)
        // .add_systems(Update, NoiseListener::update)
        .run();
}

fn setup_page(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut t = Transform::IDENTITY;
    t.rotate_y(20.);
    let page = PbrBundle {
        mesh: meshes.add(Cuboid::new(5., 5., 0.2)),
        material: materials.add(Color::WHITE),
        transform: t,
        ..Default::default()
    };
    commands.spawn(page);
}
