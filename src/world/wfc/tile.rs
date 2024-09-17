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
            Self::EMPTY => return "EMPTY".to_string(),
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
    pub const EMPTY: u8 = 0b0000_0000;
    pub const WALL: u8 = 0b0000_0001;
    pub const CORNER: u8 = 0b0000_0010;

    /// Next 3 bits are rotation info
    const ROT_MASK: u8 = 0b111_0000;
    pub const ROT_0: u8 = 0b1_0000;
    pub const ROT_90: u8 = 0b11_0000;
    pub const ROT_180: u8 = 0b101_0000;
    pub const ROT_270: u8 = 0b111_0000;

    pub const ALL_VALID_TYPES: [u8; 3] = [Self::EMPTY, Self::WALL, Self::CORNER];
    pub const ALL_VALID_ROTATIONS: [u8; 4] =
        [Self::ROT_0, Self::ROT_90, Self::ROT_180, Self::ROT_270];

    pub const ALL_VALID_IDS: LazyLock<Vec<Self>> = LazyLock::new(|| {
        TileID::ALL_VALID_TYPES
            .into_iter()
            .fold(Vec::<TileID>::new(), |mut acc, typ| {
                if typ != TileID::EMPTY {
                    acc.append(TileID::ALL_VALID_ROTATIONS.into_iter().fold(
                        &mut Vec::<TileID>::new(),
                        |inner_acc, rot| {
                            inner_acc.push((typ + rot).into());
                            inner_acc
                        },
                    ));
                } else {
                    acc.push(typ.into())
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

    pub fn connection_map(&self) -> ConnectionMap {
        let mut tile_map = HashMap::new();
        match self.0 {
            _ if self.0 == TileID::EMPTY => {
                tile_map.insert(
                    Orientation::Top,
                    ConnectionSocket::Either(Connection::Either),
                );
                tile_map.insert(
                    Orientation::Right,
                    ConnectionSocket::Either(Connection::Either),
                );
                tile_map.insert(
                    Orientation::Bottom,
                    ConnectionSocket::Either(Connection::Either),
                );
                tile_map.insert(
                    Orientation::Left,
                    ConnectionSocket::Either(Connection::Either),
                );
            }

            _ if self.0 == TileID::WALL + TileID::ROT_0 || self.0 == TileID::WALL => {
                tile_map.insert(
                    Orientation::Top,
                    ConnectionSocket::Either(Connection::First),
                );
                tile_map.insert(
                    Orientation::Right,
                    ConnectionSocket::Either(Connection::None),
                );
                tile_map.insert(
                    Orientation::Bottom,
                    ConnectionSocket::Either(Connection::First),
                );
                tile_map.insert(
                    Orientation::Left,
                    // ConnectionSocket::female(Connection::Either),
                    ConnectionSocket::Either(Connection::None),
                );
            }

            _ if self.0 == TileID::CORNER + TileID::ROT_0 || self.0 == TileID::CORNER => {
                tile_map.insert(
                    Orientation::Top,
                    ConnectionSocket::Either(Connection::None),
                    // ConnectionSocket::female(Connection::Second),
                );
                tile_map.insert(
                    Orientation::Right,
                    ConnectionSocket::Either(Connection::First),
                );
                tile_map.insert(
                    Orientation::Bottom,
                    ConnectionSocket::Either(Connection::First),
                );
                tile_map.insert(
                    Orientation::Left,
                    // ConnectionSocket::female(Connection::Second),
                    ConnectionSocket::Either(Connection::None),
                );
            }

            _ if self.0 == TileID::WALL + TileID::ROT_90 => {
                tile_map.insert(
                    Orientation::Top,
                    // ConnectionSocket::female(Connection::Either),
                    ConnectionSocket::Either(Connection::None),
                );
                tile_map.insert(
                    Orientation::Right,
                    ConnectionSocket::Either(Connection::First),
                );
                tile_map.insert(
                    Orientation::Bottom,
                    ConnectionSocket::Either(Connection::None),
                );
                tile_map.insert(
                    Orientation::Left,
                    ConnectionSocket::Either(Connection::First),
                );
            }

            _ if self.0 == TileID::CORNER + TileID::ROT_90 => {
                tile_map.insert(
                    Orientation::Top,
                    // ConnectionSocket::female(Connection::First),
                    ConnectionSocket::Either(Connection::None),
                );
                tile_map.insert(
                    Orientation::Right,
                    // ConnectionSocket::female(Connection::Second),
                    ConnectionSocket::Either(Connection::None),
                );
                tile_map.insert(
                    Orientation::Left,
                    ConnectionSocket::Either(Connection::First),
                );
                tile_map.insert(
                    Orientation::Bottom,
                    ConnectionSocket::Either(Connection::Second),
                );
            }

            _ if self.0 == TileID::WALL + TileID::ROT_180 => {
                tile_map.insert(
                    Orientation::Top,
                    ConnectionSocket::Either(Connection::Second),
                );
                tile_map.insert(
                    Orientation::Left,
                    ConnectionSocket::Either(Connection::None),
                );
                tile_map.insert(
                    Orientation::Bottom,
                    ConnectionSocket::Either(Connection::Second),
                );
                tile_map.insert(
                    Orientation::Right,
                    // ConnectionSocket::female(Connection::Either),
                    ConnectionSocket::Either(Connection::None),
                );
            }

            _ if self.0 == TileID::CORNER + TileID::ROT_180 => {
                tile_map.insert(
                    Orientation::Top,
                    ConnectionSocket::Either(Connection::Second),
                );
                tile_map.insert(
                    Orientation::Left,
                    ConnectionSocket::Either(Connection::Second),
                );
                tile_map.insert(
                    Orientation::Bottom,
                    // ConnectionSocket::female(Connection::First),
                    ConnectionSocket::Either(Connection::None),
                );
                tile_map.insert(
                    Orientation::Right,
                    // ConnectionSocket::female(Connection::First),
                    ConnectionSocket::Either(Connection::None),
                );
            }

            _ if self.0 == TileID::WALL + TileID::ROT_270 => {
                tile_map.insert(Orientation::Top, ConnectionSocket::Either(Connection::None));
                tile_map.insert(
                    Orientation::Left,
                    ConnectionSocket::Either(Connection::Second),
                );
                tile_map.insert(
                    Orientation::Right,
                    ConnectionSocket::Either(Connection::Second),
                );
                tile_map.insert(
                    Orientation::Bottom,
                    // ConnectionSocket::female(Connection::Either),
                    ConnectionSocket::Either(Connection::None),
                );
            }

            _ if self.0 == TileID::CORNER + TileID::ROT_270 => {
                tile_map.insert(
                    Orientation::Top,
                    ConnectionSocket::Either(Connection::First),
                );
                tile_map.insert(
                    Orientation::Left,
                    // ConnectionSocket::female(Connection::First),
                    ConnectionSocket::Either(Connection::None),
                );
                tile_map.insert(
                    Orientation::Right,
                    ConnectionSocket::Either(Connection::Second),
                );
                tile_map.insert(
                    Orientation::Bottom,
                    // ConnectionSocket::female(Connection::Second),
                    ConnectionSocket::Either(Connection::None),
                );
            }

            other => panic!("{other:?} is not a valid rotation"),
        }
        tile_map
    }
}

pub(super) type ConnectionMap = HashMap<Orientation, ConnectionSocket>;

pub const TILE_CONNECTION_MAP: LazyLock<HashMap<TileID, ConnectionMap>> = LazyLock::new(|| {
    let mut ret_map = HashMap::new();
    for id in LazyLock::force(&TileID::ALL_VALID_IDS) {
        ret_map.insert(*id, id.connection_map());
    }
    ret_map
});

#[derive(Debug, Hash, Copy, PartialEq, Clone)]
pub enum ConnectionSocket {
    Either(Connection),
    MaleFemale {
        male: Connection,
        female: Connection,
    },
}

#[allow(unused)]
impl ConnectionSocket {
    fn male_female(male: Connection, female: Connection) -> Self {
        Self::MaleFemale { male, female }
    }
    fn female(female: Connection) -> Self {
        Self::MaleFemale {
            male: Connection::None,
            female,
        }
    }
    fn male(male: Connection) -> Self {
        Self::MaleFemale {
            male,
            female: Connection::None,
        }
    }

    /// Checks that self will accept incoming connection. If self is MF, only checks that self's F
    /// connection matches other's M connection
    pub fn accepts_incoming_connection(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Either(con), other) => match other {
                Self::MaleFemale { male: o_m, .. } => con == o_m,
                Self::Either(ocon) => con == ocon,
            },
            (Self::MaleFemale { female, .. }, other) => match other {
                Self::MaleFemale { male: o_m, .. } => female == o_m,
                Self::Either(con) => female == con,
            },
        }
    }
}

#[derive(Debug, Hash, Copy, Clone)]
pub enum Connection {
    None,
    First,
    Second,
    Either,
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        return match (self, other) {
            (Self::Either, o) => match o {
                Self::None => false,
                _ => true,
            },
            (o, Self::Either) => match o {
                Self::None => false,
                _ => true,
            },
            (Self::First, Self::First) => true,
            (Self::Second, Self::Second) => true,
            (Self::None, Self::None) => true,
            _ => false,
        };
    }
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
    pub(super) fn invert(&self, orientation: &Orientation) -> (Orientation, Connection) {
        let o = orientation.invert();
        let c = match self {
            Self::None => Self::Either,
            Self::First => Self::Second,
            Self::Second => Self::First,
            Self::Either => Self::None,
        };
        (o, c)
    }
}
