use bevy::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InteractionExecutionType {
    ChangeState,
    ReadRoomState,
}

/// Event triggered when the player enters a new room or when a significant
/// room-related change occurs.
///
/// This event is used to trigger actions like opening the van UI or updating the
/// state of interactive objects based on the room's current state.
#[derive(Clone, Debug, Default, Event)]
pub struct RoomChangedEvent {
    /// Set to `true` if the event is triggered during level initialization.
    pub initialize: bool,
    /// Set to `true` if the van UI should be opened automatically (e.g., when the
    /// player returns to the starting area).
    pub open_van: bool,
}

impl RoomChangedEvent {
    /// Creates a new `RoomChangedEvent` specifically for level initialization.
    ///
    /// The `initialize` flag is set to `true`, and the `open_van` flag is set based on
    /// the given value.
    pub fn init(open_van: bool) -> Self {
        Self {
            initialize: true,
            open_van,
        }
    }
}
