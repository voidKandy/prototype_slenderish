use std::{fmt::Debug, sync::LazyLock};

use super::heap::{Heapable, MinHeapMap};
use bevy::{
    prelude::*,
    utils::{hashbrown::HashMap, HashSet},
};
use bevy_inspector_egui::egui::TextBuffer;
use rand::{thread_rng, Rng};

type ConnectionMap = HashMap<Orientation, Connection>;
type ConnectionKey = (Orientation, Connection);

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
enum Connection {
    None,
    First,
    Second,
    Both,
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
enum Orientation {
    Top,
    Right,
    Bottom,
    Left,
}

impl Orientation {
    fn invert(&self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Right => Self::Left,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
        }
    }
}

impl Connection {
    fn invert(&self, orientation: &Orientation) -> ConnectionKey {
        let o = orientation.invert();
        let c = match self {
            Self::None => Self::Both,
            Self::First => Self::Second,
            Self::Second => Self::First,
            Self::Both => Self::None,
        };
        (o, c)
    }
}

pub(super) type TileID = u16;
pub(super) const FLOOR: u16 = 0;
pub(super) const WALL_0: u16 = 1;
pub(super) const WALL_90: u16 = 2;
pub(super) const WALL_180: u16 = 3;
pub(super) const WALL_270: u16 = 4;
pub(super) const CORNER_0: u16 = 5;
pub(super) const CORNER_90: u16 = 6;
pub(super) const CORNER_180: u16 = 7;
pub(super) const CORNER_270: u16 = 8;

pub(super) const ALL_TILE_IDS: [TileID; 9] = [
    FLOOR, WALL_0, WALL_90, WALL_180, WALL_270, CORNER_0, CORNER_90, CORNER_180, CORNER_270,
];

fn tile_id_to_str(id: &TileID) -> String {
    match id {
        _ if id == &FLOOR => "FLOOR".to_string(),
        _ if id == &WALL_0 => "WALL_0".to_string(),
        _ if id == &WALL_90 => "WALL_90".to_string(),
        _ if id == &WALL_180 => "WALL_180".to_string(),
        _ if id == &WALL_270 => "WALL_270".to_string(),
        _ if id == &CORNER_0 => "CORNER_0".to_string(),
        _ if id == &CORNER_90 => "CORNER_90".to_string(),
        _ if id == &CORNER_180 => "CORNER_180".to_string(),
        _ if id == &CORNER_270 => "CORNER_270".to_string(),
        other => format!("Unexpected tile id: {other}"),
    }
}

// Map of all anti connections for each tile
const INVERTED_TILE_CONNECTION_MAP: LazyLock<HashMap<TileID, ConnectionMap>> =
    LazyLock::new(|| create_tile_connection_map(true));

// Map of all connections for each tile
const TILE_CONNECTION_MAP: LazyLock<HashMap<TileID, ConnectionMap>> =
    LazyLock::new(|| create_tile_connection_map(false));

#[derive(Debug, PartialEq, Clone)]
pub(super) struct Rotation(Quat);

impl From<u32> for Rotation {
    fn from(value: u32) -> Self {
        // use bevy::prelude::Quat;
        let quat = match value {
            _ if value == 0 => Quat::IDENTITY,
            _ if value == 90 => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            _ if value == 180 => Quat::from_rotation_y(std::f32::consts::PI),
            _ if value == 270 => Quat::from_rotation_y(3.0 * std::f32::consts::FRAC_PI_2),
            other => panic!("rotation cannot be built from value of {other}"),
        };
        Self(quat)
    }
}

impl AsRef<Quat> for Rotation {
    fn as_ref(&self) -> &Quat {
        &self.0
    }
}

#[derive(PartialEq, Clone, Eq)]
enum TileCellState {
    Collapsed(TileID),
    Wave(Wave),
}

impl Debug for TileCellState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Collapsed(id) => {
                write!(f, "{}", tile_id_to_str(id))
            }
            Self::Wave(wave) => {
                let mut ids_buffer = String::new();
                for id in &wave.possible_tiles {
                    ids_buffer.push_str(&format!("{}, ", tile_id_to_str(&id)));
                }
                let ids_buffer = ids_buffer.trim_end_matches(", ");
                write!(f, "Wave(Possible Tiles: [{ids_buffer}])")
            }
        }
    }
}

impl Default for TileCellState {
    fn default() -> Self {
        Self::Wave(Wave::new())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Wave {
    possible_tiles: HashSet<TileID>,
}

impl Wave {
    pub fn new() -> Self {
        let mut possible_tiles = HashSet::new();
        for id in ALL_TILE_IDS {
            possible_tiles.insert(id);
        }

        Self { possible_tiles }
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct TileCell {
    state: TileCellState,
    pub(super) x: u32,
    pub(super) y: u32,
}

#[derive(Debug)]
pub struct TileCellGrid {
    dimension_size: usize,
    heap_map: MinHeapMap<TileCell>,
}

impl TileCell {
    pub fn id(&self) -> Option<TileID> {
        if let TileCellState::Collapsed(id) = self.state {
            return Some(id);
        }
        None
    }
}

impl TileCellGrid {
    pub fn new(size: u32) -> Self {
        let mut heap_map = MinHeapMap::new();
        for y in 1..=size {
            for x in 1..=size {
                let tile = TileCell {
                    state: TileCellState::default(),
                    x,
                    y,
                };
                heap_map.insert(tile);
            }
        }
        Self {
            heap_map,
            dimension_size: size as usize,
        }
    }

    pub fn collapse_all_into_vec(&mut self) -> Vec<TileCell> {
        let mut grid_collapsed = vec![];
        while let Some(mut current) = self.heap_map.pop().ok() {
            println!("current: {current:?}");
            if current.state.collapsed() {
                break;
            }
            current.state.force_collapse();
            // println!("collapsed: {current:?}");
            let current_id = current.state.tile_id().unwrap();
            let neighbor_coords = self.neighbor_coords(&current);
            for (orient, coords) in neighbor_coords {
                if let Ok(_) = self.heap_map.lookup_and_mutate(coords, |tile_cell| {
                    tile_cell.state.update((current_id, orient.invert()));
                }) {
                    // println!("updated val at coords: {coords:?}");
                }
            }
            grid_collapsed.push(current);
        }
        grid_collapsed
    }

    fn neighbor_coords(&self, cell: &TileCell) -> Vec<(Orientation, (u32, u32))> {
        let mut neighbors = vec![];
        let (x, y) = (cell.x, cell.y);
        if x > 1 {
            neighbors.push((Orientation::Left, (x - 1, y)));
        }

        if x < self.dimension_size as u32 {
            neighbors.push((Orientation::Right, (x + 1, y)));
        }

        if y < self.dimension_size as u32 {
            neighbors.push((Orientation::Top, (x, y + 1)));
        }

        if y > 1 {
            neighbors.push((Orientation::Bottom, (x, y - 1)));
        }
        neighbors
    }
}

impl Heapable for TileCell {
    fn x(&self) -> u32 {
        self.x
    }
    fn y(&self) -> u32 {
        self.y
    }
}

impl PartialOrd for TileCell {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if let TileCellState::Wave(wave) = &self.state {
            if let TileCellState::Wave(other_wave) = &other.state {
                return wave
                    .possible_tiles
                    .len()
                    .partial_cmp(&other_wave.possible_tiles.len());
            }
        }
        Some(std::cmp::Ordering::Greater)
    }
}

impl Ord for TileCell {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if let TileCellState::Wave(wave) = &self.state {
            if let TileCellState::Wave(other_wave) = &other.state {
                return wave
                    .possible_tiles
                    .len()
                    .cmp(&other_wave.possible_tiles.len());
            }
        }
        std::cmp::Ordering::Greater
    }
}

impl TileCellState {
    fn tile_id(&self) -> Option<TileID> {
        if let TileCellState::Collapsed(id) = self {
            return Some(*id);
        }
        None
    }

    fn collapsed(&self) -> bool {
        if let TileCellState::Collapsed(_) = &self {
            return true;
        }
        false
    }

    fn force_collapse(&mut self) {
        if let TileCellState::Wave(wave) = &self {
            if wave.possible_tiles.len() == 0 {
                warn!("Had to force a tile to be a floor :(");
                *self = Self::Collapsed(FLOOR);
                return;
            }
            let mut rng = thread_rng();
            let idx = rng.gen_range(0..wave.possible_tiles.len());
            let tile = wave.possible_tiles.iter().nth(idx).unwrap();
            *self = Self::Collapsed(*tile)
        }
    }

    /// Expects to take in a neighbor orientation relative to self
    /// A neighbor to the right of self, should be passed with Orientation::Right
    fn update(
        &mut self,
        (collapsed_neighbor_id, collapsed_neighbor_orient): (TileID, Orientation),
    ) {
        if let TileCellState::Wave(ref mut wave) = self {
            // println!(
            //     "updating from collapsed neighbor: {} : {collapsed_neighbor_orient:?}",
            //     tile_id_to_str(&collapsed_neighbor_id)
            // );
            let orientation_of_self_relative_to_neighbor = collapsed_neighbor_orient.invert();
            let i = INVERTED_TILE_CONNECTION_MAP;
            let t = TILE_CONNECTION_MAP;
            let inverted_connection_map = LazyLock::force(&i);
            let connection_map = LazyLock::force(&t);

            let neighbors_connections = connection_map
                .get(&collapsed_neighbor_id)
                .expect("this should not fail");

            let mut ids_to_remove = vec![];
            let neighbor_connection = neighbors_connections
                .get(&orientation_of_self_relative_to_neighbor)
                .expect("failed to get connection at an orientation");
            // println!(
            //     "neighbor can connect: {orientation_of_self_relative_to_neighbor:?} : {neighbor_connection:?}"
            // );

            for id in &wave.possible_tiles {
                let possible_wave_connects = connection_map.get(id).expect("this should not fail");
                if let Some(connect) = possible_wave_connects.get(&collapsed_neighbor_orient) {
                    // println!(
                    //     "got connect: {collapsed_neighbor_orient:?} : {connect:?} for id: {}",
                    //     tile_id_to_str(id)
                    // );
                    if connect != neighbor_connection {
                        // println!("removing {}", tile_id_to_str(id));
                        ids_to_remove.push(id.clone());
                    } else {
                        // println!("keeping {}", tile_id_to_str(id));
                    }
                }
            }
            for id in ids_to_remove {
                // println!("removing {}", tile_id_to_str(&id));
                wave.possible_tiles.remove(&id);
            }
        }
    }
}

fn create_tile_connection_map(invert: bool) -> HashMap<TileID, ConnectionMap> {
    let insert_fn = |map: &mut ConnectionMap, or: Orientation, con: Connection| {
        if invert {
            let (k, v) = con.invert(&or);
            map.insert(k, v);
        } else {
            map.insert(or, con);
        }
    };

    let mut ret_map = HashMap::new();
    for id in ALL_TILE_IDS {
        let mut tile_map = HashMap::new();
        match id {
            _ if id == FLOOR => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::None);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::None);
                insert_fn(&mut tile_map, Orientation::Right, Connection::None);
                insert_fn(&mut tile_map, Orientation::Left, Connection::None);
            }
            _ if id == WALL_0 => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::First);
                insert_fn(&mut tile_map, Orientation::Right, Connection::None);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::First);
                insert_fn(&mut tile_map, Orientation::Left, Connection::None);
            }
            _ if id == CORNER_0 => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::None);
                insert_fn(&mut tile_map, Orientation::Right, Connection::First);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::First);
                insert_fn(&mut tile_map, Orientation::Left, Connection::None);
            }
            _ if id == WALL_90 => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::None);
                insert_fn(&mut tile_map, Orientation::Right, Connection::First);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::None);
                insert_fn(&mut tile_map, Orientation::Left, Connection::First);
            }
            _ if id == CORNER_90 => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::None);
                insert_fn(&mut tile_map, Orientation::Right, Connection::None);
                insert_fn(&mut tile_map, Orientation::Left, Connection::First);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::Second);
            }
            _ if id == WALL_180 => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::Second);
                insert_fn(&mut tile_map, Orientation::Left, Connection::None);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::Second);
                insert_fn(&mut tile_map, Orientation::Right, Connection::None);
            }
            _ if id == CORNER_180 => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::Second);
                insert_fn(&mut tile_map, Orientation::Left, Connection::Second);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::None);
                insert_fn(&mut tile_map, Orientation::Right, Connection::None);
            }
            _ if id == WALL_270 => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::None);
                insert_fn(&mut tile_map, Orientation::Left, Connection::Second);
                insert_fn(&mut tile_map, Orientation::Right, Connection::Second);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::None);
            }
            _ if id == CORNER_270 => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::First);
                insert_fn(&mut tile_map, Orientation::Left, Connection::None);
                insert_fn(&mut tile_map, Orientation::Right, Connection::Second);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::None);
            }
            other => panic!("{other:?} is not a valid rotation"),
        }
        ret_map.insert(id, tile_map);
    }
    ret_map
}

mod tests {
    #![allow(unused_imports)]
    use std::{mem::take, sync::LazyLock};

    use bevy::reflect::List;
    use rand::{rngs::ThreadRng, Rng};

    use crate::world::{
        heap::MinHeapMap,
        wave::{
            Connection, Orientation, TileCellGrid, TileCellState, CORNER_180, CORNER_90, WALL_90,
        },
    };

    use super::{TileCell, WALL_0, WALL_180};

    #[test]
    fn update_tile_cell_works() {
        let mut cellstate = TileCellState::default();

        let neighbor1 = (WALL_0, Orientation::Top);
        let neighbor2 = (WALL_90, Orientation::Right);
        let neighbor3 = (WALL_0, Orientation::Bottom);
        cellstate.update(neighbor1);
        cellstate.update(neighbor2);
        cellstate.update(neighbor3);

        let expected = WALL_0;
        if let TileCellState::Collapsed(v) = cellstate {
            assert_eq!(v, expected)
        }
    }
}
