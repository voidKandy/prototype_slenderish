use bevy::prelude::*;
use bevy_rapier3d::rapier::crossbeam::epoch::Pointable;

use crate::items::equip::world::cube::WorldCubeBundle;

#[derive(Event, Debug)]
pub enum EquipItemEvent {
    Spawned(u8),
    PickedUp(u8),
    Dropped(u8),
}

impl AsRef<u8> for EquipItemEvent {
    fn as_ref(&self) -> &u8 {
        match self {
            Self::Spawned(i) => i,
            Self::PickedUp(i) => i,
            Self::Dropped(i) => i,
        }
    }
}

// impl EquipItemEvent {
//     pub const SPHERE: u8 = 1;
//     pub const CUBE: u8 = 2;
//
//     fn handle(mut ev_equip_item: EventReader<EquipItemEvent>) {
//         for ev in ev_equip_item.read() {
//             warn!("equip item event: {ev:?}");
//
//             let item: &u8 = ev.as_ref();
//
//             match item {
//                 Self::CUBE => {
//                     match ev {
//                         EquipItemEvent::Dropped(_) => {
//                             warn!("Dropped item: {item:?}");
//                             item
//                         }
//                         EquipItemEvent::PickedUp(_) => {
//                             warn!("PickedUp item: {item:?}");
//                             item
//                         }
//                         EquipItemEvent::Spawned(_) => {
//                             warn!("Spawned item: {item:?}");
//                             item
//                         }
//                     };
//                 }
//                 Self::SPHERE => {
//                     match ev {
//                         EquipItemEvent::Dropped(_) => {
//                             warn!("Dropped item: {item:?}");
//                             item
//                         }
//                         EquipItemEvent::PickedUp(_) => {
//                             warn!("PickedUp item: {item:?}");
//                             item
//                         }
//                         EquipItemEvent::Spawned(_) => {
//                             warn!("Spawned item: {item:?}");
//                             item
//                         }
//                     };
//                 }
//             }
//         }
//     }
// }
