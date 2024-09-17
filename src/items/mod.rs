use bevy::app::Plugin;
use equip::EquipItemPlugin;
use pickup::PickupItemPlugin;

pub mod equip;
pub mod pickup;

#[derive(Debug)]
pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        EquipItemPlugin.build(app);
        PickupItemPlugin.build(app);
    }
}
