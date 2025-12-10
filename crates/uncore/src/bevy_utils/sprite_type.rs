#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, Reflect)]
pub enum SpriteType {
    #[default]
    Sprite,
    Ghostly,
    UvReactive,
    RedLightReactive,
    GhostOrb, // New SpriteType for Ghost Orbs
    Player,   // Added Player explicitly from context of maplight.rs usage
    Breach,   // Added Breach explicitly
    Miasma,   // Added Miasma explicitly
    Other,    // Added Other explicitly
}
