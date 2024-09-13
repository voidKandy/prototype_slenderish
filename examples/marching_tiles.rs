#[path = "../examples/common/lib.rs"]
mod common;
use std::{cell::LazyCell, sync::LazyLock};

use bevy::prelude::*;
use prototype_slenderish::world::{
    chunks2::{MarchingTileBundle, TILE_ID_MARCHING_TILES_MAP},
    wave::TileCellGrid,
    Heapable, GROUND_Y,
};

pub fn main() {
    let mut app = common::test_app();
    app.add_systems(Startup, setup_marching_tiles).run();
}

fn setup_marching_tiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tiles = TileCellGrid::new(3).collapse_all_into_vec();
    let t = TILE_ID_MARCHING_TILES_MAP;
    let map = LazyLock::force(&t);
    let origin = Transform::from_xyz(-10., -50., -20.);

    warn!("tiles: {tiles:?}");

    for (i, tile) in tiles.iter().enumerate() {
        let marching = map.get(&tile.id().unwrap()).unwrap();
        let (mesh, transform) =
            MarchingTileBundle::marching_tile_mesh(&origin, marching, tile.x(), tile.y());

        let bundle = PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::srgb(0., 0., 0.5)),
            transform,
            ..Default::default()
        };
        commands.spawn((Name::new(format!("tile {i}")), bundle));
    }

    // let wall = MarchingTileBundle::wall_mesh();
    // let bundle = PbrBundle {
    //     mesh: meshes.add(wall),
    //     material: materials.add(Color::srgb(0., 0., 0.5)),
    //     transform: Transform::from_xyz(0., 0., 0.),
    //     ..Default::default()
    // };
    // commands.spawn((Name::new("wall"), bundle));
}
