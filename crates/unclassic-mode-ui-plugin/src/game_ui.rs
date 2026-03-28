use super::gear_ui::{setup_ui_gear_inv_left, setup_ui_gear_inv_right};
use bevy::ui::BackgroundColor;
use bevy::ui::widget::ImageNode;
use bevy::{color::palettes::css, prelude::*};
use bevy_persistent::Persistent;
use unbehavior_core::behavior::Behavior;
use unclassic_mode_core::colors;
use uncommon_app_core::platform::plt::{FONT_SCALE, UI_SCALE};
use ungear_core::assets::GearAssets;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::ui::EvidenceUI;
use uninput_core::states::InGameUiState;
use unmission_core::resources::MissionConcludingCinematic;
use unorchestrator_core::UIContextState;
use unplayer_core::components::{MainPlayer, PlayerSpectating, PlayerSprite};
use unsettings_core::game::GameplaySettings;
use unvitals_core::components::PlayerVitals;
use unwalkie_core::components::WalkieText;

use crate::assets::GameUiAssets;

/// Marker for the fade-to-black overlay that plays during the mission concluding cinematic.
#[derive(Component)]
struct MissionFadeOverlay;

#[derive(Component, Debug)]
struct GameUI;

#[derive(Component, Debug, PartialEq, Eq)]
enum ElementObjectUI {
    Name,
    Description,
    Grab,
}

#[derive(Component, Debug)]
struct DamageBackground {
    exp: f32,
}

impl DamageBackground {
    fn new(exp: f32) -> Self {
        Self { exp }
    }
}

#[derive(Component, Debug)]
#[allow(dead_code)]
struct HeldObjectUI;

#[derive(Component, Debug)]
struct RightSideGearUI;

#[derive(Component, Debug, Default)]
struct WalkieTextUIRoot;

fn update_damage_vignette_color(
    qp: Query<(&PlayerVitals, Has<PlayerSpectating>), With<MainPlayer>>,
    mut qb: Query<(
        Option<&mut ImageNode>,
        &mut BackgroundColor,
        &DamageBackground,
    )>,
) {
    for (player_vitals, is_spectating) in &qp {
        if is_spectating {
            // Spectator visual effect (desaturated/blue tint)
            for (mut o_uiimage, mut bgcolor, _dmg) in &mut qb {
                // Ignore dmg.exp for spectator, use fixed visual
                let dst_color = Color::srgba(0.0, 0.0, 0.2, 0.4);
                let old_color = o_uiimage.as_ref().map(|x| x.color).unwrap_or(bgcolor.0);
                let new_color = lerp_color(old_color, dst_color, 0.1);
                if old_color != new_color {
                    if let Some(uiimage) = o_uiimage.as_mut() {
                        uiimage.color = new_color;
                    } else {
                        bgcolor.0 = new_color;
                    }
                }
            }
        } else {
            let health = (player_vitals.health.clamp(0.0, 100.0) / 100.0).clamp(0.0, 1.0);
            let crazyness = (1.0 - player_vitals.sanity / 100.0).clamp(0.0, 1.0);
            for (mut o_uiimage, mut bgcolor, dmg) in &mut qb {
                let rhealth = (1.0 - health).powf(dmg.exp);
                let crazyness = crazyness.powf(dmg.exp);
                let alpha = ((rhealth * 10.0).clamp(0.0, 0.3) + rhealth.powi(2) * 0.7 + crazyness)
                    .clamp(0.0, 1.0);
                let rhealth2 = (1.0 - alpha * 0.9).clamp(0.0001, 1.0);
                let red = f32::tanh(rhealth * 2.0).clamp(0.0, 1.0) * rhealth2;
                let dst_color = Color::srgba(red, 0.0, 0.0, alpha);
                let old_color = o_uiimage.as_ref().map(|x| x.color).unwrap_or(bgcolor.0);
                let new_color = lerp_color(old_color, dst_color, 0.2);
                if old_color != new_color {
                    if let Some(uiimage) = o_uiimage.as_mut() {
                        uiimage.color = new_color;
                    } else {
                        bgcolor.0 = new_color;
                    }
                }
            }
        }
    }
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::srgba(
        a.to_srgba().red * (1.0 - t) + b.to_srgba().red * t,
        a.to_srgba().green * (1.0 - t) + b.to_srgba().green * t,
        a.to_srgba().blue * (1.0 - t) + b.to_srgba().blue * t,
        a.to_srgba().alpha * (1.0 - t) + b.to_srgba().alpha * t,
    )
}

fn cleanup(
    mut commands: Commands,
    qg: Query<Entity, With<GameUI>>,
    qwt: Query<Entity, With<WalkieTextUIRoot>>,
) {
    for gui in qg.iter() {
        commands.entity(gui).despawn();
    }
    for gui in qwt.iter() {
        commands.entity(gui).despawn();
    }
}

fn pause(mut qg: Query<&mut Visibility, With<GameUI>>) {
    for mut vis in qg.iter_mut() {
        *vis = Visibility::Hidden;
    }
}

fn resume(mut qg: Query<&mut Visibility, With<GameUI>>) {
    for mut vis in qg.iter_mut() {
        *vis = Visibility::Visible;
    }
}

fn setup_ui(
    mut commands: Commands,
    ui_assets: Res<GameUiAssets>,
    gear_assets: Res<GearAssets>,
    game_settings: Res<Persistent<GameplaySettings>>,
) {
    commands
        .spawn((
            WalkieTextUIRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Percent(20.0),
                width: Val::Percent(60.0),
                height: Val::Auto,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(100),
        ))
        .with_children(|parent| {
            parent
                .spawn(Text::new(""))
                .insert(TextFont {
                    font: ui_assets.font_chakra_italic.clone(),
                    font_size: 18.0 * FONT_SCALE,
                    ..default()
                })
                .insert(TextLayout::new_with_justify(Justify::Center))
                .insert(BackgroundColor(css::BLACK.with_alpha(0.6).into()))
                .insert(TextColor(colors::WALKIE_TALKIE_COLOR))
                .insert(Node {
                    padding: UiRect::axes(Val::Px(10.0 * UI_SCALE), Val::Px(1.0 * UI_SCALE)),
                    ..default()
                })
                .insert(Visibility::Hidden)
                .insert(WalkieText);
        });

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(Pickable::IGNORE)
        .insert(ZIndex(-5))
        .insert(BackgroundColor(css::BLACK.with_alpha(0.0).into()))
        .insert(GameUI)
        .insert(DamageBackground::new(4.0));
    // Full-screen fade-to-black overlay, driven by MissionConcludingCinematic.
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(Pickable::IGNORE)
        .insert(ZIndex(500))
        .insert(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)))
        .insert(GameUI)
        .insert(MissionFadeOverlay);
    commands
        .spawn(ImageNode {
            image: ui_assets.vignette.clone(),
            color: Color::NONE,
            ..default()
        })
        .insert(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(Pickable::IGNORE)
        .insert(ZIndex(-4))
        .insert(GameUI)
        .insert(DamageBackground::new(0.7));

    type Cb<'a, 'b> = &'b mut ChildSpawnerCommands<'a>;
    let key_legend = |p: Cb| {
        let ch_control = game_settings.character_controls.to_string();
        let controls = vec![
            format!("[{ch_control}]: Movement"),
            "[Shift]: Sprint".to_string(),
            "[Ctrl]: Left Hand".to_string(),
            "[E]: Interact".to_string(),
            "[F]: Grab/Move".to_string(),
            "[G]: Drop".to_string(),
            "[Q]: Next".to_string(),
            "[T]: Swap Hands".to_string(),
            "[C]: Change Evidence".to_string(),
        ];
        for ctrl in controls {
            p.spawn(Node {
                padding: UiRect::all(Val::Px(6.0 * UI_SCALE)),
                margin: UiRect::right(Val::Px(6.0 * UI_SCALE)),
                ..default()
            })
            .insert(BackgroundColor(css::BLACK.with_alpha(0.3).into()))
            .with_children(|p| {
                p.spawn(Text::new(ctrl))
                    .insert(TextFont {
                        font: ui_assets.font_chakra_light.clone(),
                        font_size: 16.0 * FONT_SCALE,
                        ..default()
                    })
                    .insert(TextLayout::new_with_justify(Justify::Center))
                    .insert(TextColor(colors::INVENTORY_STATS_COLOR))
                    .insert(Node {
                        align_self: AlignSelf::Center,
                        justify_self: JustifySelf::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::bottom(Val::Px(-6.0 * UI_SCALE)),
                        padding: UiRect::all(Val::Px(4.0 * UI_SCALE)),
                        ..default()
                    });
            });
        }
    };
    let evidence = |p: Cb| setup_ui_evidence(p, &ui_assets);
    let inv_left = |p: Cb| setup_ui_gear_inv_left(p, &ui_assets, &gear_assets);
    let inv_right = |p: Cb| setup_ui_gear_inv_right(p, &ui_assets, &gear_assets);
    let bottom_panel = |p: Cb| {
        p.spawn(Node {
            min_width: Val::Px(100.0 * UI_SCALE),
            max_width: Val::Percent(33.3),
            align_content: AlignContent::Start,
            flex_direction: FlexDirection::Row,
            border: UiRect::all(Val::Px(1.0 * UI_SCALE)),
            padding: UiRect::all(Val::Px(1.0)),
            flex_grow: 0.01,
            flex_shrink: 0.0,
            ..Default::default()
        })
        .insert(colors::DEBUG_BCOLOR)
        .insert(BackgroundColor(colors::PANEL_BGCOLOR))
        .with_children(inv_left);

        p.spawn(Node {
            border: UiRect::all(Val::Px(1.0 * UI_SCALE)),
            padding: UiRect::all(Val::Px(8.0 * UI_SCALE)),
            flex_grow: 1.0,
            ..Default::default()
        })
        .insert(colors::DEBUG_BCOLOR)
        .insert(BackgroundColor(colors::PANEL_BGCOLOR))
        .with_children(evidence);

        p.spawn(Node {
            flex_direction: FlexDirection::Column,
            max_width: Val::Percent(33.3),
            align_items: AlignItems::Start,
            align_content: AlignContent::Center,
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::all(Val::Px(1.0)),
            flex_grow: 0.01,
            ..Default::default()
        })
        .insert(colors::DEBUG_BCOLOR)
        .insert(BackgroundColor(colors::PANEL_BGCOLOR))
        .with_children(|p| {
            p.spawn(Node {
                align_items: AlignItems::Start,
                align_content: AlignContent::Center,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(1.0)),
                flex_grow: 1.0,
                ..Default::default()
            })
            .insert(RightSideGearUI)
            .with_children(inv_right);
        });
    };
    let game_ui = |p: Cb| {
        p.spawn(Node {
            height: Val::Percent(5.0),
            min_height: Val::Px(16.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            ..default()
        })
        .insert(colors::DEBUG_BCOLOR)
        .insert(Pickable::IGNORE)
        .with_children(|parent| {
            parent
                .spawn(ImageNode {
                    image: ui_assets.title.clone(),
                    ..default()
                })
                .insert(Node {
                    aspect_ratio: Some(130.0 / 17.0),
                    width: Val::Percent(20.0),
                    height: Val::Auto,
                    max_width: Val::Percent(20.0),
                    max_height: Val::Percent(100.0),
                    flex_shrink: 1.0,
                    flex_grow: 0.0,
                    ..default()
                });
            parent.spawn(Node {
                flex_grow: 0.5,
                ..default()
            });
        });

        p.spawn(Node {
            min_height: Val::Px(2.0),
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::all(Val::Px(1.0)),
            flex_grow: 1.0,
            ..Default::default()
        })
        .insert(Pickable::IGNORE)
        .insert(colors::DEBUG_BCOLOR);

        p.spawn(Node {
            align_content: AlignContent::Start,
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            border: UiRect::all(Val::Px(1.0 * UI_SCALE)),
            padding: UiRect::all(Val::Px(6.0 * UI_SCALE)),
            flex_grow: 0.0,
            ..Default::default()
        })
        .insert(BackgroundColor(colors::PANEL_BGCOLOR))
        .with_children(key_legend);

        p.spawn(Node {
            height: Val::Px(100.0 * UI_SCALE),
            width: Val::Percent(99.9),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(6.0),
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::all(Val::Px(1.0)),
            ..Default::default()
        })
        .insert(colors::DEBUG_BCOLOR)
        .insert(BackgroundColor(colors::PANEL_BGCOLOR))
        .with_children(bottom_panel);
    };

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Column,
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::all(Val::Px(1.0)),
            ..default()
        })
        .insert(Pickable::IGNORE)
        .insert(colors::DEBUG_BCOLOR)
        .insert(GameUI)
        .with_children(game_ui);
    debug!("Game UI loaded");
}

fn setup_ui_evidence(parent: &mut ChildSpawnerCommands, ui_assets: &GameUiAssets) {
    parent
        .spawn((
            Text::default(),
            TextFont {
                font: ui_assets.font_chakra_regular.clone(),
                font_size: 22.0 * FONT_SCALE,
                ..default()
            },
            TextColor(colors::INVENTORY_STATS_COLOR.with_alpha(1.0)),
            TextLayout::default(),
            Node::default(),
            EvidenceUI,
        ))
        .with_children(|parent| {
            parent
                .spawn(TextSpan::new("Freezing temps:"))
                .insert(TextFont {
                    font: ui_assets.font_chakra_regular.clone(),
                    font_size: 22.0 * FONT_SCALE,
                    ..default()
                 })
                .insert(TextColor(colors::INVENTORY_STATS_COLOR.with_alpha(1.0)));
            parent
                .spawn(TextSpan::new(" [+] Evidence Found\n"))
                .insert(TextFont {
                    font: ui_assets.font_victor_semibold.clone(),
                    font_size: 20.0 * FONT_SCALE,
                    ..default()
                })
                .insert(TextColor(css::GREEN.with_alpha(1.0).into()));
            parent
                .spawn(TextSpan::new(
                    "The ghost and the breach will make the ambient colder.\nSome ghosts will make the temperature drop below 0.0ºC.",
                ))
                .insert(TextFont {
                    font: ui_assets.font_chakra_light.clone(),
                    font_size: 20.0 * FONT_SCALE,
                    ..default()
                 })
                .insert(TextColor(colors::INVENTORY_STATS_COLOR));
        });
}

fn tick_mission_fade(
    cinematic: Option<Res<MissionConcludingCinematic>>,
    mut q_overlay: Query<&mut BackgroundColor, With<MissionFadeOverlay>>,
) {
    let alpha = cinematic
        .map(|c| c.timer.elapsed_secs())
        .unwrap_or(0.0)
        .tanh()
        .max(0.0);
    for mut color in q_overlay.iter_mut() {
        color.0 = Color::srgba(0.0, 0.0, 0.0, alpha.sqrt());
    }
}

fn toggle_held_object_ui(
    mut text_query: Query<(&mut Text, &mut TextColor, &ElementObjectUI)>,
    players: Query<&PlayerGear, (With<PlayerSprite>, With<MainPlayer>)>,
    objects: Query<&Behavior>,
) {
    if let Ok(player_gear) = players.single()
        && let Some(held_object) = &player_gear.held_item
        && let Ok(behavior) = objects.get(held_object.entity)
    {
        for (mut text, _, _) in text_query
            .iter_mut()
            .filter(|(_, _, e)| **e == ElementObjectUI::Name)
        {
            text.0.clone_from(&behavior.p.object.name);
        }

        for (mut text, _, _) in text_query
            .iter_mut()
            .filter(|(_, _, e)| **e == ElementObjectUI::Description)
        {
            text.0 = "Object Description".into();
        }

        for (mut text, mut color, _) in text_query
            .iter_mut()
            .filter(|(_, _, e)| **e == ElementObjectUI::Grab)
        {
            if behavior.p.object.movable {
                text.0 = "[Grab]: Move Object".into();
                color.0 = colors::INVENTORY_STATS_COLOR;
            } else {
                text.0 = "[Grab]: -".into();
                color.0 = colors::INVENTORY_STATS_COLOR.with_alpha(0.3);
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(UIContextState::InGame), setup_ui)
        .add_systems(OnExit(UIContextState::InGame), cleanup)
        .add_systems(OnEnter(InGameUiState::Running), resume)
        .add_systems(OnExit(InGameUiState::Running), pause)
        .add_systems(
            Update,
            (
                toggle_held_object_ui.run_if(in_state(InGameUiState::Running)),
                update_damage_vignette_color,
            )
                .run_if(in_state(InGameUiState::Running)),
        )
        .add_systems(
            Update,
            tick_mission_fade.run_if(in_state(UIContextState::InGame)),
        );
}
