use bevy::prelude::*;
// Update imports to use unwalkiecore
use unwalkiecore::{WalkieEvent, WalkiePlay};

// Import the new triggers module and its contents
use crate::triggers::locomotion_interaction::{
    BoardData, // Placeholder Resource
    BumpingInDarknessCollisionTracker,
    // BumpingInDarkness
    BumpingInDarknessConfig,
    DoorHesitationTimer,
    // DoorInteractionHesitation
    DoorInteractionHesitationConfig,
    // ErraticMovementEarly
    ErraticMovementEarlyConfig,
    ErraticMovementTrackers,
    // General
    GameState,
    GrabDropActionTracker,
    HideUnhideTracker,
    LevelReadyTimer,
    NearPickableTimer,
    PlayerActionEvent,   // Placeholder Event
    PlayerInteractEvent, // Placeholder Event
    // PlayerStuckAtStart
    PlayerSpawnPoint,
    PlayerStuckAtStartConfig,
    RoomDB,               // Placeholder Resource
    StaticCollisionEvent, // Placeholder Event
    // StrugglingWithGrabDrop
    StrugglingWithGrabDropConfig,
    // StrugglingWithHideUnhide
    StrugglingWithHideUnhideConfig,
    VanArea,
    VanCollisionEvent, // Placeholder Event
    bumping_in_darkness_system,
    door_interaction_hesitation_system,
    erratic_movement_early_system,
    player_stuck_at_start_system,
    struggling_with_grab_drop_system,
    struggling_with_hide_unhide_system,
    // Placeholder Components (not directly used in plugin.rs but part of the module)
    // PlayerSprite, PlayerInputDirection, MainEntranceDoor, TileState, PlayerGear, PickableObject, PlayerViewConeTarget, HeldItem, GhostSprite, HidingSpot, Hiding
};

pub struct UnhaunterWalkiePlugin;

impl Plugin for UnhaunterWalkiePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WalkieEvent>().init_resource::<WalkiePlay>();

        // Initialize placeholder resources (ideally these are part of a core game plugin)
        // For now, initializing them here to make the walkie plugin self-contained.
        app.init_resource::<GameState>();
        app.init_resource::<PlayerSpawnPoint>();
        app.init_resource::<VanArea>();
        app.init_resource::<LevelReadyTimer>(); // Used by multiple systems
        app.init_resource::<BoardData>(); // For BumpingInDarkness
        app.init_resource::<RoomDB>(); // For BumpingInDarkness

        // Initialize Config resources for locomotion_interaction systems
        app.init_resource::<PlayerStuckAtStartConfig>();
        app.init_resource::<ErraticMovementEarlyConfig>();
        app.init_resource::<DoorInteractionHesitationConfig>();
        app.init_resource::<BumpingInDarknessConfig>();
        app.init_resource::<StrugglingWithGrabDropConfig>();
        app.init_resource::<StrugglingWithHideUnhideConfig>();

        // Initialize Tracker/Timer resources for locomotion_interaction systems
        app.init_resource::<ErraticMovementTrackers>();
        app.init_resource::<DoorHesitationTimer>();
        app.init_resource::<BumpingInDarknessCollisionTracker>();
        app.init_resource::<NearPickableTimer>();
        app.init_resource::<GrabDropActionTracker>();
        app.init_resource::<HideUnhideTracker>();

        // Register placeholder Event types
        app.add_event::<VanCollisionEvent>();
        app.add_event::<StaticCollisionEvent>();
        app.add_event::<PlayerActionEvent>();
        app.add_event::<PlayerInteractEvent>();

        // Add the core walkie talk/playback system
        crate::walkie_play::app_setup(app);

        // Add systems from triggers::locomotion_interaction
        app.add_systems(Update, player_stuck_at_start_system);
        app.add_systems(Update, erratic_movement_early_system);
        app.add_systems(Update, door_interaction_hesitation_system);
        app.add_systems(Update, bumping_in_darkness_system);
        app.add_systems(Update, struggling_with_grab_drop_system);
        app.add_systems(Update, struggling_with_hide_unhide_system);

        // TODO: Add systems from triggers::environmental_awareness
        // ... etc. for all other categories
    }
}
