use super::{
    heap_map::{Heapable, MinHeapMap},
    tile::{Orientation, TileID, TILE_CONNECTION_MAP},
};
use bevy::{prelude::*, utils::HashSet};
use rand::{thread_rng, Rng};
use std::{fmt::Debug, sync::LazyLock};

#[derive(Debug, PartialEq, Clone)]
pub struct Rotation(Quat);

impl From<u32> for Rotation {
    fn from(value: u32) -> Self {
        // use bevy::prelude::Quat;
        let quat = match value {
            _ if value == 0 => Quat::IDENTITY,
            // 90 degree rotation is negative becayse rotating on the y creates a counter clockwise
            //  rotation, this makes it clockwise
            _ if value == 90 => Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
            _ if value == 180 => Quat::from_rotation_y(std::f32::consts::PI),
            _ if value == 270 => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
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
enum WaveGridCell {
    Collapsed { cell: TileCell, x: u32, y: u32 },
    Wave { wave: Wave, x: u32, y: u32 },
}

impl Heapable for WaveGridCell {
    fn x(&self) -> u32 {
        match self {
            Self::Wave { x, .. } => *x,
            Self::Collapsed { x, .. } => *x,
        }
    }
    fn y(&self) -> u32 {
        match self {
            Self::Wave { y, .. } => *y,
            Self::Collapsed { y, .. } => *y,
        }
    }
}
impl Debug for WaveGridCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Collapsed { cell, .. } => {
                write!(f, "{}", &cell.id.to_string())
            }
            Self::Wave { wave, .. } => {
                let mut ids_buffer = String::new();
                for id in &wave.possible {
                    ids_buffer.push_str(&format!("{}, ", &id.to_string()));
                }
                let ids_buffer = ids_buffer.trim_end_matches(", ");
                write!(f, "Wave(Possible Tiles: [{ids_buffer}])")
            }
        }
    }
}

impl PartialOrd for WaveGridCell {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if let WaveGridCell::Wave { wave, .. } = &self {
            if let WaveGridCell::Wave {
                wave: other_wave, ..
            } = &other
            {
                return wave.possible.len().partial_cmp(&other_wave.possible.len());
            }
        }
        Some(std::cmp::Ordering::Less)
    }
}

impl Ord for WaveGridCell {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if let WaveGridCell::Wave { wave, .. } = &self {
            if let WaveGridCell::Wave {
                wave: other_wave, ..
            } = &other
            {
                return wave.possible.len().cmp(&other_wave.possible.len());
            }
        }
        std::cmp::Ordering::Less
    }
}

impl WaveGridCell {
    fn new(x: u32, y: u32) -> Self {
        Self::Wave {
            wave: Wave::new(),
            x,
            y,
        }
    }

    fn tile_id(&self) -> Option<TileID> {
        if let Self::Collapsed { cell, .. } = self {
            return Some(cell.id);
        }
        None
    }

    fn collapse_into(&mut self, tile_id: impl Into<TileID>) {
        if let Self::Wave { x, y, .. } = self {
            *self = Self::Collapsed {
                cell: TileCell::new(tile_id.into(), *x, *y),
                x: *x,
                y: *y,
            }
        }
    }

    fn force_collapse(&mut self) {
        if let Self::Wave { ref wave, .. } = self {
            let mut rng = thread_rng();
            let mut idx = rng.gen_range(0..wave.possible.len());
            if wave.possible.len() > 1 {
                if let Some(empty_idx) = wave
                    .possible
                    .iter()
                    .position(|id| id == &TileID::EMPTY.into())
                {
                    while idx == empty_idx {
                        idx = rng.gen_range(0..wave.possible.len());
                    }
                }
            }

            let tile = wave.possible.iter().nth(idx).unwrap();
            self.collapse_into(*tile);
        }
    }

    fn collapsed(&self) -> bool {
        if let WaveGridCell::Collapsed { .. } = &self {
            return true;
        }
        false
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Wave {
    possible: HashSet<TileID>,
}

impl Wave {
    pub fn new() -> Self {
        let mut possible = HashSet::new();
        for id in LazyLock::force(&TileID::ALL_VALID_IDS) {
            possible.insert(*id);
        }
        Self { possible }
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct TileCell {
    pub id: TileID,
    pub x: u32,
    pub z: u32,
}

#[derive(Debug)]
pub struct WaveGrid {
    dimension_size: usize,
    heap_map: MinHeapMap<WaveGridCell>,
}

impl TileCell {
    fn try_from_wave_grid_cell(wgc: WaveGridCell) -> Option<Self> {
        if let WaveGridCell::Collapsed { cell, x, y } = wgc {
            return Some(Self {
                id: cell.id,
                x,
                z: y,
            });
        }
        None
    }
    fn new(id: TileID, x: u32, z: u32) -> Self {
        Self { id, x, z }
    }

    pub fn local_transform(&self, mesh_size: f32) -> Transform {
        let mut transform = Transform::IDENTITY;
        let translation = mesh_size / 2.0 - 0.5;
        match self.id.type_value() {
            // TileID::FLOOR => {
            // transform.translation.y -= translation;
            // transform.translation.z += translation + 0.5;
            // transform.translation.x += translation + 0.5;
            // }
            _ => {
                if let Some(rot) = self.id.rotation_identity() {
                    match &rot {
                        _ if rot == 0.into() => {
                            transform.translation.z -= translation;
                        }
                        _ if rot == 90.into() => {
                            transform.translation.x += translation;
                        }
                        _ if rot == 180.into() => {
                            transform.translation.z += translation;
                        }
                        _ if rot == 270.into() => {
                            transform.translation.x -= translation;
                        }
                        other => panic!("unhandled rotation: {other:?}"),
                    }
                    warn!("rotating tile by: {rot:?}");
                    transform.rotate(rot.0);
                }
            }
        }

        transform
    }
}

impl WaveGrid {
    pub fn new(size: u32) -> Self {
        let mut heap_map = MinHeapMap::new();
        for z in 1..=size {
            for x in 1..=size {
                let tile = WaveGridCell::new(x, z);
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
            if current.collapsed() {
                break;
            }
            let tries_allowed = 3;
            match current {
                WaveGridCell::Collapsed { .. } => break,
                WaveGridCell::Wave { x, y, .. } => {
                    current.force_collapse();
                    // println!("collapsed: {current:?}");
                    let neighbor_coords = self.neighbor_coords(x, y);
                    for (orient, coords) in neighbor_coords {
                        if let Ok(_) = self.heap_map.lookup_and_mutate(coords, |state| {
                            if let WaveGridCell::Wave { wave, .. } = state {
                                wave.update(
                                    (current.tile_id().unwrap(), orient.invert()),
                                    tries_allowed,
                                );
                            }
                        }) {
                            // println!("updated val at coords: {coords:?}");
                        }
                    }
                    grid_collapsed.push(TileCell::try_from_wave_grid_cell(current).unwrap());
                }
            }
        }
        grid_collapsed
    }

    fn neighbor_coords(&self, x: u32, y: u32) -> Vec<(Orientation, (u32, u32))> {
        let mut neighbors = vec![];
        if x > 1 {
            // neighbors.push((Orientation::Bottom, (x - 1, y)));
            neighbors.push((Orientation::Left, (x - 1, y)));
        }

        if x < self.dimension_size as u32 {
            neighbors.push((Orientation::Right, (x + 1, y)));
            // neighbors.push((Orientation::Top, (x + 1, y)));
        }

        if y < self.dimension_size as u32 {
            neighbors.push((Orientation::Top, (x, y + 1)));
            // neighbors.push((Orientation::Right, (x, y + 1)));
        }

        if y > 1 {
            neighbors.push((Orientation::Bottom, (x, y - 1)));
            // neighbors.push((Orientation::Left, (x, y - 1)));
        }
        neighbors
    }
}

impl Wave {
    /// Expects to take in a neighbor orientation relative to self
    /// A neighbor to the right of self, should be passed with Orientation::Right
    fn update(
        &mut self,
        (collapsed_neighbor_id, collapsed_neighbor_orient): (impl Into<TileID>, Orientation),
        // How many times can the update attempt to collapse before moving on?
        tries_allowed: usize,
    ) {
        let orientation_of_self_relative_to_neighbor = collapsed_neighbor_orient.invert();
        let t = TILE_CONNECTION_MAP;
        let connection_map = LazyLock::force(&t);
        let neighbor_id: TileID = collapsed_neighbor_id.into();

        let neighbors_connections = connection_map
            .get(&neighbor_id)
            .expect("this should not fail");

        let mut ids_to_remove = vec![];
        let neighbor_connection = neighbors_connections
            .get(&orientation_of_self_relative_to_neighbor)
            .expect("failed to get connection at an orientation");
        warn!(
            r#"
            tileid of neighbor: {}
            self is {orientation_of_self_relative_to_neighbor:?} of neighbor.
            self can connect at {collapsed_neighbor_orient:?} at {neighbor_connection:?}"#,
            neighbor_id.to_string()
        );

        warn!("iterating through possibilites of self");

        for id in &self.possible {
            let possible_self_connects = connection_map.get(id).expect("this should not fail");
            if let Some(connect) = possible_self_connects.get(&collapsed_neighbor_orient) {
                warn!(
                    "got connect: {collapsed_neighbor_orient:?} : {connect:?} for id: {}",
                    id.to_string()
                );
                if !connect.accepts_incoming_connection(neighbor_connection) {
                    warn!("removing {}", id.to_string());
                    ids_to_remove.push(id.clone());
                }
            }
            if ids_to_remove.len() >= tries_allowed {
                break;
            }
        }

        for id in ids_to_remove {
            if self.possible.len() == 1 {
                warn!(
                    "could not remove all, will collapse into {}",
                    id.to_string()
                );
                return;
            }
            self.possible.remove(&id);
        }
    }
}
