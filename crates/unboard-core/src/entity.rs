use bevy::prelude::*;

/// Marker component for game sprites (generic game objects)
#[derive(Component, Debug)]
pub struct GameSprite;

/// Marker component for map tile sprites (static tileset elements)
#[derive(Component, Debug)]
pub struct MapTileSprite;

/// Component categorizing entity sprite type for rendering/behavior decisions
#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub enum SpriteType {
    Ghost,
    GhostOrb,
    Breach,
    Player,
    Miasma,
    #[default]
    Other,
}

/// Component that stores the upscale factor of the asset (e.g., 3.0 for zoom03x).
/// Used to downscale the Transform so the object maintains its intended size.
#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct ResolutionFactor(pub f32);

impl Default for ResolutionFactor {
    fn default() -> Self {
        Self(1.0)
    }
}

impl ResolutionFactor {
    pub fn ratio(&self) -> f32 {
        1.0 / self.0
    }
}
