use bevy::prelude::*;
use uncareer_core::grade::Grade;

use crate::assets::CampaignAssets;

/// Utility for creating badge UI elements in the map hub
pub(crate) struct BadgeUtils;

impl BadgeUtils {
    /// Creates a UI element displaying a grade badge
    ///
    /// If grade is NA, no badge will be shown unless show_na is true
    pub(crate) fn create_badge(
        parent: &mut ChildSpawnerCommands,
        campaign_assets: &CampaignAssets,
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
                    image: campaign_assets.badges.clone(),
                    texture_atlas: Some(TextureAtlas {
                        index: grade.badge_index(),
                        layout: campaign_assets.badges_layout.clone(),
                    }),
                    ..default()
                },
                Node {
                    width: Val::Px(size),
                    height: Val::Px(size),
                    ..default()
                },
            ))
            .insert(Pickable {
                should_block_lower: false,
                ..default()
            })
            .id();

        Some(entity)
    }
}
