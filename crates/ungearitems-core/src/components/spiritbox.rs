use bevy::prelude::*;

/// A component representing the Spirit Box gear item.
/// This device scans radio frequencies and can sometimes pick up paranormal vocal phenomena.
#[derive(Component, Debug, Clone, Default)]
pub struct SpiritBox {
    /// A frame counter used for animating the display sprites.
    pub mode_frame: u32,
    /// True if the ghost is currently providing a direct response through the box.
    /// This is set to true only for legitimate ghost answers, not for interference.
    pub ghost_answer: bool,
    /// The timestamp (`time.elapsed_secs()`) of the last significant state change,
    /// used to time animations and response logic.
    pub last_change_secs: f32,
    /// Accumulates when conditions are right (darkness, proximity to ghost, low temp).
    /// When this reaches a threshold, a ghost response can be triggered.
    pub charge: f32,
    /// True if the UI hint for acknowledging the evidence should be blinking.
    /// This is used to draw the player's attention to new evidence.
    pub blinking_hint_active: bool,
}
