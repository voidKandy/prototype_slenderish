use super::grid::Rotation;
use bevy::{
    log::warn,
    utils::{hashbrown::HashMap, tracing::instrument},
};
use std::{fmt::Debug, ops::Add, sync::LazyLock};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TileID(u8);

impl From<u8> for TileID {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl ToString for TileID {
    fn to_string(&self) -> String {
        let typ = match self.0 & Self::TILE_TYPE_MASK {
            Self::FLOOR => return "FLOOR".to_string(),
            Self::WALL => "WALL_",
            Self::CORNER => "CORNER_",
            _ => "UNDEFINED_",
        };
        let rot = self.rotation_identity().unwrap_or(0.into());
        let rot = match rot {
            _ if rot == 0.into() => "0",
            _ if rot == 90.into() => "90",
            _ if rot == 180.into() => "180",
            _ if rot == 270.into() => "270",
            other => panic!("invalid rotation: {other:?}"),
        };
        format!("{typ}{rot}")
    }
}

impl Add for TileID {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl TileID {
    /// First Four bits are terrain type info
    const TILE_TYPE_MASK: u8 = 0b0000_1111;
    pub const FLOOR: u8 = 0b0000_0000;
    pub const WALL: u8 = 0b0000_0001;
    pub const CORNER: u8 = 0b0000_0010;

    /// Next 3 bits are rotation info
    const ROT_MASK: u8 = 0b111_0000;
    pub const ROT_0: u8 = 0b1_0000;
    pub const ROT_90: u8 = 0b11_0000;
    pub const ROT_180: u8 = 0b101_0000;
    pub const ROT_270: u8 = 0b111_0000;

    pub const ALL_VALID_TYPES: [u8; 3] = [Self::FLOOR, Self::WALL, Self::CORNER];
    pub const ALL_VALID_ROTATIONS: [u8; 4] =
        [Self::ROT_0, Self::ROT_90, Self::ROT_180, Self::ROT_270];

    pub const ALL_VALID_IDS: LazyLock<Vec<Self>> = LazyLock::new(|| {
        TileID::ALL_VALID_TYPES
            .into_iter()
            .fold(Vec::<TileID>::new(), |mut acc, typ| {
                acc.push(typ.into());
                if typ != TileID::FLOOR {
                    acc.append(TileID::ALL_VALID_ROTATIONS.into_iter().fold(
                        &mut Vec::<TileID>::new(),
                        |inner_acc, rot| {
                            inner_acc.push((typ + rot).into());
                            inner_acc
                        },
                    ));
                }
                acc
            })
    });

    pub fn type_value(&self) -> u8 {
        self.0 & Self::TILE_TYPE_MASK
    }

    /// For now FLOOR has no rotations, potentially could be used later for pitch (ramps)?
    pub fn rotation_identity(&self) -> Option<Rotation> {
        let id = self.0 & Self::ROT_MASK;
        match id {
            0b000 => return Some(0.into()),
            Self::ROT_0 => return Some(0.into()),
            Self::ROT_90 => return Some(90.into()),
            Self::ROT_180 => return Some(180.into()),
            Self::ROT_270 => return Some(270.into()),
            // This could map onto something else entirely.
            // Material, shader, etc
            0b011 | 0b001 | 0b010 => {
                warn!("rotation identity in an unhandled case");
                return None;
            }
            other => panic!("pretty sure this is exaustive.. {other}"),
        }
    }

    pub fn connection_map(&self, invert: bool) -> ConnectionMap {
        let mut tile_map = HashMap::new();
        let insert_fn = |map: &mut ConnectionMap, or: Orientation, con: Connection| {
            if invert {
                let (k, v) = con.invert(&or);
                map.insert(k, v);
            } else {
                map.insert(or, con);
            }
        };
        let id = self.0 & Self::TILE_TYPE_MASK;
        match id {
            _ if id == TileID::FLOOR => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::None);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::None);
                insert_fn(&mut tile_map, Orientation::Right, Connection::None);
                insert_fn(&mut tile_map, Orientation::Left, Connection::None);
            }
            _ if id == TileID::WALL + TileID::ROT_0 || id == TileID::WALL => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::First);
                insert_fn(&mut tile_map, Orientation::Right, Connection::None);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::First);
                insert_fn(&mut tile_map, Orientation::Left, Connection::None);
            }
            _ if id == TileID::CORNER + TileID::ROT_0 || id == TileID::CORNER => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::None);
                insert_fn(&mut tile_map, Orientation::Right, Connection::First);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::First);
                insert_fn(&mut tile_map, Orientation::Left, Connection::None);
            }
            _ if id == TileID::WALL + TileID::ROT_90 => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::None);
                insert_fn(&mut tile_map, Orientation::Right, Connection::First);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::None);
                insert_fn(&mut tile_map, Orientation::Left, Connection::First);
            }
            _ if id == TileID::CORNER + TileID::ROT_90 => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::None);
                insert_fn(&mut tile_map, Orientation::Right, Connection::None);
                insert_fn(&mut tile_map, Orientation::Left, Connection::First);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::Second);
            }
            _ if id == TileID::WALL + TileID::ROT_180 => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::Second);
                insert_fn(&mut tile_map, Orientation::Left, Connection::None);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::Second);
                insert_fn(&mut tile_map, Orientation::Right, Connection::None);
            }
            _ if id == TileID::CORNER + TileID::ROT_180 => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::Second);
                insert_fn(&mut tile_map, Orientation::Left, Connection::Second);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::None);
                insert_fn(&mut tile_map, Orientation::Right, Connection::None);
            }
            _ if id == TileID::WALL + TileID::ROT_270 => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::None);
                insert_fn(&mut tile_map, Orientation::Left, Connection::Second);
                insert_fn(&mut tile_map, Orientation::Right, Connection::Second);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::None);
            }
            _ if id == TileID::CORNER + TileID::ROT_270 => {
                insert_fn(&mut tile_map, Orientation::Top, Connection::First);
                insert_fn(&mut tile_map, Orientation::Left, Connection::None);
                insert_fn(&mut tile_map, Orientation::Right, Connection::Second);
                insert_fn(&mut tile_map, Orientation::Bottom, Connection::None);
            }
            other => panic!("{other:?} is not a valid rotation"),
        }
        tile_map
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Connection {
    None,
    First,
    Second,
    Both,
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Orientation {
    Top,
    Right,
    Bottom,
    Left,
}

impl Orientation {
    pub(super) fn invert(&self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Right => Self::Left,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
        }
    }
}

impl Connection {
    pub(super) fn invert(&self, orientation: &Orientation) -> ConnectionKey {
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

pub(super) type ConnectionMap = HashMap<Orientation, Connection>;
type ConnectionKey = (Orientation, Connection);

const INVERTED_TILE_CONNECTION_MAP: LazyLock<HashMap<TileID, ConnectionMap>> =
    LazyLock::new(|| create_tile_connection_map(true));

pub const TILE_CONNECTION_MAP: LazyLock<HashMap<TileID, ConnectionMap>> =
    LazyLock::new(|| create_tile_connection_map(false));

fn create_tile_connection_map(invert: bool) -> HashMap<TileID, ConnectionMap> {
    let mut ret_map = HashMap::new();

    for id in LazyLock::force(&TileID::ALL_VALID_IDS) {
        warn!("id: {} is in all valid", id.to_string());
        ret_map.insert(*id, id.connection_map(invert));
    }
    ret_map
}
