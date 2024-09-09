use bevy::{
    color::palettes::css::WHITE, input::keyboard::KeyboardInput, prelude::*,
    render::view::RenderLayers, utils::HashSet,
};

use super::equip_item::{
    player::{EquipItemPlayer, EquipItemPlayerBundle},
    EquipItemMaterial,
};
pub const VIEW_MODEL_RENDER_LAYER: usize = 1;

#[derive(Component, Default)]
pub struct PlayerViewModel {
    pub currently_equipped: Option<EquipItemPlayer>,
    pub other_items: HashSet<EquipItemPlayer>,
}

#[derive(Bundle)]
pub struct PlayerViewModelBundle {
    player: PlayerViewModel,
    name: Name,
    camera: Camera3dBundle,
    render_layers: RenderLayers,
}

impl PlayerViewModelBundle {
    pub fn new() -> Self {
        let name = Name::new("View Model Camera");
        let render_layers = RenderLayers::layer(VIEW_MODEL_RENDER_LAYER);
        let camera = Camera3dBundle {
            camera: Camera {
                order: 1,
                ..default()
            },
            projection: PerspectiveProjection {
                fov: 70.0_f32.to_radians(),
                ..default()
            }
            .into(),
            ..default()
        };
        Self {
            player: PlayerViewModel::default(),
            name,
            camera,
            render_layers,
        }
    }
}

#[derive(Component)]
pub struct LookingAtText;

pub fn single_text_sections(str: &str) -> Vec<TextSection> {
    vec![TextSection {
        value: str.to_owned(),
        style: TextStyle {
            font_size: 12.0,
            ..Default::default()
        },
    }]
}

impl PlayerViewModel {
    /// Adds an item to equipment, equipping it if something already isnt
    pub fn add_to_equipment(&mut self, item: impl Into<EquipItemPlayer>) {
        match self.currently_equipped {
            Some(_) => {
                let _ = self.other_items.insert(item.into());
            }
            None => self.currently_equipped = Some(item.into()),
        }
    }

    pub fn cycle_equipment_next(&mut self) {
        if let Some(next) = self.other_items.iter().next() {
            let taken = self
                .other_items
                .take(&next.clone())
                .expect("Should be valid get");

            if let Some(item) = self.currently_equipped.take() {
                let _ = self.other_items.insert(item);
            }
            self.currently_equipped = Some(taken);
        }
    }
}

pub fn update_player_equipment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<EquipItemMaterial>>,

    mut player_vm_q: Query<(Entity, &mut PlayerViewModel), With<PlayerViewModel>>,
    equip_item_q: Query<(Entity, &EquipItemPlayer), With<EquipItemPlayer>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let (player_vm_entity, mut player_vm) = player_vm_q.single_mut();

    if keys.just_pressed(KeyCode::KeyN) {
        player_vm.cycle_equipment_next();
    }

    if let Some(item) = player_vm.currently_equipped {
        for (equip_item_entity, equip_item) in &equip_item_q {
            if *equip_item != item {
                commands.entity(equip_item_entity).despawn();
            }
        }
        let player_item =
            EquipItemPlayerBundle::from_equip_item(item.into(), &mut meshes, &mut materials);
        commands.entity(player_vm_entity).with_children(|p| {
            p.spawn(player_item);
        });
    }
}

pub fn spawn_player_hud(commands: &mut Commands) {
    let parent_node = NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ..default()
    };

    let crosshair = NodeBundle {
        style: Style {
            width: Val::Px(2.5),
            height: Val::Px(2.5),
            align_self: AlignSelf::Center,
            position_type: PositionType::Relative,
            ..default()
        },
        background_color: WHITE.into(),
        ..default()
    };

    let currently_looking_at = TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "".to_owned(),
                style: TextStyle {
                    font_size: 12.0,
                    ..Default::default()
                },
            }],
            justify: JustifyText::Center,
            ..Default::default()
        },
        style: Style {
            margin: UiRect {
                top: Val::Vh(2.),
                ..Default::default()
            },
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            ..Default::default()
        },
        ..Default::default()
    };

    commands.spawn(parent_node).with_children(|p| {
        p.spawn(crosshair);
        p.spawn((currently_looking_at, LookingAtText));
    });
}
