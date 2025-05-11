use bevy::prelude::*;
use uncore::types::grade::Grade;
use uncore::types::root::game_assets::GameAssets;

/// Utility for creating badge UI elements in the map hub
pub struct BadgeUtils;

impl BadgeUtils {
    /// Creates a UI element displaying a grade badge
    ///
    /// If grade is NA, no badge will be shown unless show_na is true
    pub fn create_badge(
        parent: &mut ChildBuilder,
        handles: &GameAssets,
        grade: Grade,
        size: f32,
        show_na: bool,
    ) -> Option<Entity> {
        // Don't show badge for NA unless explicitly requested
        if grade == Grade::NA && !show_na {
            return None;
        }

        // Create the badge using the proper Bevy 0.15 component structure
        let entity = parent
            .spawn((
                // ImageNode with texture_atlas as a field, not a separate component
                ImageNode {
                    image: handles.images.badges.clone(),
                    texture_atlas: Some(TextureAtlas {
                        index: grade.badge_index(),
                        layout: handles.images.badges_atlas.clone(),
                    }),
                    ..default()
                },
                Node {
                    width: Val::Px(size),
                    height: Val::Px(size),
                    ..default()
                },
            ))
            .insert(PickingBehavior {
                should_block_lower: false,
                ..default()
            })
            .id();

        Some(entity)
    }

    /// Creates a UI element displaying a grade badge with a text label
    ///
    /// If grade is NA, no badge will be shown unless show_na is true
    pub fn create_badge_with_label(
        parent: &mut ChildBuilder,
        handles: &GameAssets,
        grade: Grade,
        size: f32,
        show_na: bool,
    ) -> Option<Entity> {
        // Don't show badge for NA unless explicitly requested
        if grade == Grade::NA && !show_na {
            return None;
        }

        // Create a container node with badge and label as children
        let entity = parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|p| {
                // Create the badge image
                BadgeUtils::create_badge(p, handles, grade, size, true);

                // Add the text description using proper Bevy 0.15 components
                p.spawn((
                    Text::new(grade.description()),
                    TextFont {
                        font: handles.fonts.chakra.w400_regular.clone(),
                        font_size: size * 0.75,
                        font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                    },
                    TextColor(grade.color()),
                    Node {
                        margin: UiRect::left(Val::Px(8.0)),
                        ..default()
                    },
                ));
            })
            .id();

        Some(entity)
    }
}
