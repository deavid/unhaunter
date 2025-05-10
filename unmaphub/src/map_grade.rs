use bevy::prelude::*;
use uncore::types::root::game_assets::GameAssets;

/// Represents available grades that maps can be assigned
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapGrade {
    /// Best performance, excellent completion
    A,
    /// Very good performance
    B,
    /// Good performance
    C,
    /// Passing performance
    D,
    /// Failed performance
    F,
}

impl Default for MapGrade {
    fn default() -> Self {
        MapGrade::F
    }
}

impl MapGrade {
    /// Get the badge index for this grade in the badge spritesheet
    pub fn badge_index(&self) -> usize {
        match self {
            MapGrade::A => 0,
            MapGrade::B => 1,
            MapGrade::C => 2,
            MapGrade::D => 3,
            MapGrade::F => 4,
        }
    }

    /// Get a color that represents this grade
    pub fn color(&self) -> Color {
        match self {
            MapGrade::A => Color::rgb(0.0, 0.8, 0.0), // Green
            MapGrade::B => Color::rgb(0.5, 0.8, 0.0), // Light green
            MapGrade::C => Color::rgb(1.0, 0.8, 0.0), // Yellow
            MapGrade::D => Color::rgb(1.0, 0.5, 0.0), // Orange
            MapGrade::F => Color::rgb(1.0, 0.0, 0.0), // Red
        }
    }

    /// Get a textual description of this grade
    pub fn description(&self) -> &'static str {
        match self {
            MapGrade::A => "Excellent",
            MapGrade::B => "Very Good",
            MapGrade::C => "Good",
            MapGrade::D => "Passing",
            MapGrade::F => "Failed",
        }
    }
}

/// Helper struct for creating badge UI elements
pub struct MapBadgeBuilder;

impl MapBadgeBuilder {
    /// Creates a UI element displaying a grade badge
    pub fn create_badge(
        parent: &mut ChildBuilder,
        handles: &GameAssets,
        grade: MapGrade,
        size: f32,
    ) -> Entity {
        parent
            .spawn(ImageBundle {
                image: UiImage {
                    texture: handles.images.badges.clone(),
                    flip_x: false,
                    flip_y: false,
                },
                texture_atlas: Some(TextureAtlas {
                    index: grade.badge_index(),
                    layout: handles.images.badges_atlas.clone(),
                }),
                style: Style {
                    width: Val::Px(size),
                    height: Val::Px(size),
                    ..default()
                },
                ..default()
            })
            .id()
    }

    /// Creates a UI element displaying a grade badge with a text label
    pub fn create_badge_with_label(
        parent: &mut ChildBuilder,
        handles: &GameAssets,
        grade: MapGrade,
        size: f32,
    ) -> Entity {
        parent
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|p| {
                MapBadgeBuilder::create_badge(p, handles, grade, size);

                p.spawn(
                    TextBundle::from_section(
                        grade.description(),
                        TextStyle {
                            font: handles.fonts.chakra.w400_regular.clone(),
                            font_size: size * 0.75,
                            color: grade.color(),
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::left(Val::Px(8.0)),
                        ..default()
                    }),
                );
            })
            .id()
    }
}
