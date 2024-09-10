use bevy::{color::palettes::css::WHITE, prelude::*, render::view::RenderLayers};

pub const VIEW_MODEL_RENDER_LAYER: usize = 1;

#[derive(Component, Default)]
pub struct PlayerViewModel;

#[derive(Bundle)]
pub struct PlayerViewModelBundle {
    player: PlayerViewModel,
    visibility: VisibilityBundle,
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
            visibility: VisibilityBundle::default(),
            name,
            camera,
            render_layers,
        }
    }
}

#[derive(Component)]
pub struct LookingAtText;
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
            position_type: PositionType::Absolute,
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
