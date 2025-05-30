use super::uibutton::TruckButtonType; // Assuming TruckUIGhostGuess and uibutton are still relevant here
use bevy::prelude::*;
use uncore::colors;
use uncore::components::truck::TruckUIGhostGuess;
use uncore::difficulty::CurrentDifficulty; // Use CurrentDifficulty
use uncore::platform::plt::{FONT_SCALE, UI_SCALE};
use uncore::types::evidence::Evidence;
use uncore::types::root::game_assets::GameAssets;

const MARGIN_PERCENT: f32 = 0.5 * UI_SCALE;

pub fn setup_journal_ui(
    p: &mut ChildBuilder,
    handles: &GameAssets,
    difficulty: &CurrentDifficulty,
) {
    // Journal contents
    p.spawn((
        Text::new("Select evidence:"),
        TextColor(colors::TRUCKUI_TEXT_COLOR),
        TextFont {
            font: handles.fonts.chakra.w300_light.clone(),
            font_size: 25.0 * FONT_SCALE,
            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
        },
        Node {
            margin: UiRect::all(Val::Px(4.0 * UI_SCALE)),
            ..default()
        },
    ));

    // Evidence selection
    p.spawn(Node {
        justify_content: JustifyContent::FlexStart,
        row_gap: Val::Px(4.0 * UI_SCALE),
        column_gap: Val::Px(4.0 * UI_SCALE),
        display: Display::Grid,
        grid_template_columns: vec![
            GridTrack::auto(),
            GridTrack::auto(),
            GridTrack::auto(),
            GridTrack::auto(),
        ],
        grid_template_rows: vec![GridTrack::auto(), GridTrack::auto()],
        ..default()
    })
    .with_children(|evblock| {
        for evidence in Evidence::all() {
            evblock
                .spawn(Button)
                .insert(Node {
                    min_width: Val::Px(0.0),
                    min_height: Val::Px(20.0 * UI_SCALE),
                    border: UiRect::all(Val::Px(0.9)),
                    justify_content: JustifyContent::Center,
                    display: Display::Grid,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Percent(MARGIN_PERCENT)),
                    padding: UiRect::all(Val::Px(4.0 * UI_SCALE)),
                    ..default()
                })
                .insert(BackgroundColor(Color::NONE))
                .insert(BorderColor(Color::NONE))
                .insert(Interaction::None)
                .insert(TruckButtonType::Evidence(evidence).into_component())
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(evidence.name()),
                        TextFont {
                            font: handles.fonts.titillium.w400_regular.clone(),
                            font_size: 18.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        },
                        TextColor(colors::TRUCKUI_TEXT_COLOR),
                        TextLayout::default(), // Ensure TextLayout is added
                    ));
                });
        }
    });

    // ---- Ghost guess
    p.spawn(Node {
        margin: UiRect::all(Val::Px(4.0 * UI_SCALE)),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::End,
        column_gap: Val::Percent(MARGIN_PERCENT),
        flex_basis: Val::Px(50.0 * UI_SCALE),
        flex_grow: 0.5,
        flex_shrink: 0.0,
        ..default()
    })
    .with_children(|guess| {
        guess.spawn((
            Text::new("Possible ghost with the selected evidence:"),
            TextFont {
                font: handles.fonts.chakra.w300_light.clone(),
                font_size: 25.0 * FONT_SCALE,
                font_smoothing: bevy::text::FontSmoothing::AntiAliased,
            },
            TextColor(colors::TRUCKUI_TEXT_COLOR),
            Node {
                flex_grow: 1.0,
                flex_shrink: 1.0,
                ..default()
            },
        ));
    });

    // Ghost selection
    p.spawn(Node {
        justify_content: JustifyContent::FlexStart,
        display: Display::Grid,
        grid_template_columns: vec![
            // Define columns for the grid
            GridTrack::auto(),
            GridTrack::auto(),
            GridTrack::auto(),
            GridTrack::auto(),
            GridTrack::auto(),
        ],
        grid_auto_rows: GridTrack::auto(), // Rows will adjust height automatically
        row_gap: Val::Px(2.0 * UI_SCALE),  // Reduced gap
        column_gap: Val::Px(2.0 * UI_SCALE), // Reduced gap
        flex_grow: 1.0,
        padding: UiRect::all(Val::Px(2.0 * UI_SCALE)), // Added padding
        ..default()
    })
    .insert(BackgroundColor(colors::TRUCKUI_BGCOLOR))
    .with_children(|ghost_selection| {
        // Use difficulty.0 (DifficultyStruct) to get the ghost_set
        for ghost_type in difficulty.0.ghost_set.as_vec() {
            ghost_selection
                .spawn(Button)
                .insert(Node {
                    min_width: Val::Px(0.0),
                    min_height: Val::Px(18.0 * UI_SCALE), // Slightly reduced height
                    border: UiRect::all(Val::Px(0.9)),
                    justify_content: JustifyContent::Center,
                    padding: UiRect::new(Val::Px(4.0), Val::Px(2.0), Val::Px(0.0), Val::Px(2.0)), // Adjusted padding
                    display: Display::Flex, // Changed to Flex for better centering
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .insert(BackgroundColor(Color::NONE))
                .insert(BorderColor(Color::NONE))
                .insert(Interaction::None)
                .insert(TruckButtonType::Ghost(ghost_type).into_component())
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(ghost_type.name()),
                        TextFont {
                            font: handles.fonts.titillium.w400_regular.clone(),
                            font_size: 18.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        },
                        TextColor(colors::TRUCKUI_TEXT_COLOR),
                        TextLayout::default(), // Ensure TextLayout is added
                    ));
                });
        }
    });

    // --- Ghost selected
    p.spawn(Node {
        margin: UiRect::all(Val::Px(4.0 * UI_SCALE)),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::End,
        column_gap: Val::Percent(MARGIN_PERCENT),
        flex_basis: Val::Px(50.0 * UI_SCALE),
        flex_grow: 0.5,
        flex_shrink: 0.0,
        ..default()
    })
    .with_children(|guess| {
        guess.spawn((
            Text::new("With the above evidence we believe the ghost is:"),
            TextFont {
                font: handles.fonts.chakra.w300_light.clone(),
                font_size: 25.0 * FONT_SCALE,
                font_smoothing: bevy::text::FontSmoothing::AntiAliased,
            },
            TextColor(colors::TRUCKUI_TEXT_COLOR),
            Node {
                flex_grow: 1.0,
                flex_shrink: 1.0,
                ..default()
            },
        ));
        let ghost_guess = (
            Text::new("-- Unknown --"),
            TextFont {
                font: handles.fonts.titillium.w600_semibold.clone(),
                font_size: 20.0 * FONT_SCALE,
                font_smoothing: bevy::text::FontSmoothing::AntiAliased,
            },
            TextColor(colors::TRUCKUI_TEXT_COLOR),
        );
        guess
            .spawn(Node {
                padding: UiRect::all(Val::Px(4.0 * UI_SCALE)),
                flex_basis: Val::Px(300.0 * UI_SCALE),
                flex_grow: 0.0,
                flex_shrink: 0.0,
                justify_content: JustifyContent::Center,
                ..default()
            })
            .insert(BackgroundColor(colors::TRUCKUI_BGCOLOR))
            .with_children(|node| {
                node.spawn(ghost_guess).insert(TruckUIGhostGuess);
            });
    });

    // ---- Synthesis of Unhaunter essence
    p.spawn(Button)
        .insert(Node {
            min_width: Val::Px(0.0),
            max_width: Val::Percent(60.0),
            min_height: Val::Px(30.0 * UI_SCALE),
            border: UiRect::all(Val::Px(4.0 * UI_SCALE)),
            margin: UiRect::all(Val::Px(30.0 * UI_SCALE)).with_left(Val::Percent(20.0)),
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            position_type: PositionType::Relative,
            ..default()
        })
        .insert(ZIndex(20))
        .insert(BackgroundColor(Color::NONE))
        .insert(BorderColor(Color::NONE))
        .insert(Interaction::None)
        .insert(TruckButtonType::CraftRepellent.into_component())
        .with_children(|btn| {
            btn.spawn((
                Text::new("Craft Unhaunter™ Ghost Repellent"),
                TextFont {
                    font: handles.fonts.titillium.w600_semibold.clone(),
                    font_size: 23.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                },
                TextColor(colors::TRUCKUI_TEXT_COLOR),
                TextLayout::default(),
                Node {
                    margin: UiRect::px(
                        5.0 * UI_SCALE,
                        5.0 * UI_SCALE,
                        20.0 * UI_SCALE,
                        20.0 * UI_SCALE,
                    ),
                    ..default()
                },
            ));
        });
}
