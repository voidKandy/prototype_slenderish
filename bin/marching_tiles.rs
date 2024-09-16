#[path = "../bin/common/lib.rs"]
mod common;

use bevy::{prelude::*, reflect::Array};
use prototype_slenderish::world::{
    chunks2::MarchingTileBundle,
    wave::{
        grid::{TileCell, WaveGrid},
        tile::TileID,
    },
};

pub fn main() {
    let mut app = common::test_app();
    app.add_systems(
        Startup,
        (
            // test_positions,
            // test_hand_placed
            test_grid,
        ),
    )
    .run();
}

fn test_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut grid = WaveGrid::new(9);
    let origin = Transform::IDENTITY;
    let all_cells = grid.collapse_all_into_vec();
    for (i, cell) in all_cells.into_iter().enumerate() {
        let mesh = MarchingTileBundle::cell_mesh(&cell);
        let global_transform = MarchingTileBundle::global_transform(&cell, &origin);

        let bundle = PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::srgb(0., 0., 0.5)),
            transform: global_transform.into(),
            // global_transform: transform.global,
            // transform: transform.local,
            ..Default::default()
        };
        commands.spawn((Name::new(format!("cell_{i}")), bundle));
    }
}

fn test_hand_placed(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cells = vec![
        TileCell {
            id: (TileID::CORNER + TileID::ROT_0).into(),
            x: 1,
            z: 1,
        },
        TileCell {
            id: (TileID::CORNER + TileID::ROT_90).into(),
            x: 2,
            z: 1,
        },
        TileCell {
            id: (TileID::CORNER + TileID::ROT_270).into(),
            x: 1,
            z: 2,
        },
        TileCell {
            id: (TileID::CORNER + TileID::ROT_180).into(),
            x: 2,
            z: 2,
        },
    ];

    let origin = Transform::IDENTITY;

    for cell in cells.into_iter() {
        let mesh = MarchingTileBundle::cell_mesh(&cell);
        let global_transform = MarchingTileBundle::global_transform(&cell, &origin);

        let bundle = PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::srgb(0., 0., 0.5)),
            // global_transform: transform.global,
            transform: global_transform.into(),
            ..Default::default()
        };
        commands.spawn((Name::new(cell.id.to_string()), bundle));
    }
}

fn test_positions(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tiles =
        TileID::ALL_VALID_ROTATIONS
            .into_iter()
            .fold(Vec::<TileID>::new(), |mut acc, rot| {
                acc.push((TileID::CORNER + rot).into());
                acc
            });

    let origin = Transform::from_xyz(0., 0., 0.);

    for (i, id) in tiles.into_iter().enumerate() {
        let cell = TileCell { id, x: 1, z: 1 };

        let mesh = MarchingTileBundle::cell_mesh(&cell);
        let global_transform = MarchingTileBundle::global_transform(&cell, &origin);

        let bundle = PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::srgb(0., 0., 0.5)),
            // global_transform: transform.global,
            transform: global_transform.into(),
            ..Default::default()
        };
        commands.spawn((Name::new(id.to_string()), bundle));
    }
    let floor = TileCell {
        id: TileID::FLOOR.into(),
        x: 1,
        z: 1,
    };
    let mesh = MarchingTileBundle::cell_mesh(&floor);
    let global_transform = MarchingTileBundle::global_transform(&floor, &origin);

    let bundle = PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::srgb(0., 0., 0.5)),
        transform: global_transform.into(),
        ..Default::default()
    };
    commands.spawn((Name::new("FLOOR"), bundle));
}
