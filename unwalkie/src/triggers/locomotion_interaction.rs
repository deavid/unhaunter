use bevy::prelude::*;
use unwalkiecore::{events::WalkieEvent, resources::WalkiePlay};

// --- Placeholder Types (Ideally imported from other crates like uncore, unplayer) ---
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub enum GameState {
    #[default]
    None, // Active gameplay in level
    Menu,
    Paused,
    Truck,
    // ... other states
}

#[derive(Component)]
pub struct PlayerSprite;

#[derive(Resource, Default)]
pub struct PlayerSpawnPoint(pub Vec2); // Or use Transform if Z-axis matters for spawn

#[derive(Resource, Default)]
pub struct VanArea {
    pub center: Vec2,
    pub radius: f32,
}

// Timer to track time since level became ready for player interaction
#[derive(Resource, Deref, DerefMut, Default)]
pub struct LevelReadyTimer(pub Timer); // Should be started/reset on LevelReadyEvent

// --- Configuration for PlayerStuckAtStart ---
#[derive(Resource)]
pub struct PlayerStuckAtStartConfig {
    pub time_threshold_seconds: f32,
    pub radius_threshold: f32,
}

impl Default for PlayerStuckAtStartConfig {
    fn default() -> Self {
        Self {
            time_threshold_seconds: 25.0, // Default: 25 seconds
            radius_threshold: 5.0,        // Default: 5 units radius
        }
    }
}

/// System to detect if the player hasn't moved significantly from the start for a while.
pub fn player_stuck_at_start_system(
    mut walkie_play: ResMut<WalkiePlay>,
    time: Res<Time>,
    level_ready_timer: Option<Res<LevelReadyTimer>>,
    game_state: Option<Res<GameState>>,
    player_query: Query<&Transform, With<PlayerSprite>>,
    // Using PlayerSpawnPoint as the primary way, VanArea as fallback.
    // One of these should be configured and present.
    player_spawn_point: Option<Res<PlayerSpawnPoint>>,
    van_area: Option<Res<VanArea>>, // Could be used if spawn point is less defined
    config: Res<PlayerStuckAtStartConfig>,
) {
    let current_game_state = match game_state {
        Some(gs) => gs,
        None => {
            // If GameState resource isn't present, we can't proceed.
            // This might indicate an issue with app setup or that the game isn't fully loaded.
            // error!("GameState resource not found for player_stuck_at_start_system");
            return;
        }
    };

    if *current_game_state != GameState::None {
        return; // System should only run during active gameplay in the level
    }

    let level_timer = match level_ready_timer {
        Some(lrt) => lrt,
        None => {
            // error!("LevelReadyTimer resource not found for player_stuck_at_start_system");
            return;
        }
    };

    if level_timer.paused()
        || level_timer.finished()
        || level_timer.elapsed_secs() <= config.time_threshold_seconds
    {
        // Timer not active, not yet passed threshold, or already finished (if it's not repeating)
        return;
    }

    let player_transform = match player_query.get_single() {
        Ok(transform) => transform,
        Err(_) => {
            // No player found or multiple players.
            // error!("PlayerSprite not found or multiple found for player_stuck_at_start_system");
            return;
        }
    };
    let player_position = player_transform.translation.truncate();

    let initial_pos_or_area_center;
    let effective_radius_threshold;

    if let Some(spawn_res) = player_spawn_point {
        initial_pos_or_area_center = spawn_res.0;
        effective_radius_threshold = config.radius_threshold;
    } else if let Some(van_area_res) = van_area {
        initial_pos_or_area_center = van_area_res.center;
        effective_radius_threshold = van_area_res.radius; // Use VanArea's own radius if spawn point is not specific
    } else {
        // Neither PlayerSpawnPoint nor VanArea is defined. Cannot determine if player is stuck.
        // error!("Neither PlayerSpawnPoint nor VanArea configured for player_stuck_at_start_system");
        return;
    }

    if player_position.distance(initial_pos_or_area_center) <= effective_radius_threshold {
        // Player is within the threshold radius of the starting point/area.
        walkie_play.set(
            WalkieEvent::PlayerStuckAtStart,
            time.elapsed().as_secs_f64(),
        );
    }
}

// --- Configuration and Trackers for ErraticMovementEarly ---
#[derive(Resource)]
pub struct ErraticMovementEarlyConfig {
    pub time_threshold_early_seconds: f32, // e.g., 30-45s
    pub direction_change_window_seconds: f32,
    pub direction_change_count_threshold: usize,
    pub low_displacement_threshold: f32,
    pub van_collision_window_seconds: f32,
    pub van_collision_count_threshold: usize,
}

impl Default for ErraticMovementEarlyConfig {
    fn default() -> Self {
        Self {
            time_threshold_early_seconds: 40.0,
            direction_change_window_seconds: 5.0, // Check direction changes over last 5s
            direction_change_count_threshold: 8,  // More than 8 significant direction changes
            low_displacement_threshold: 2.0,      // Moved less than 2 units
            van_collision_window_seconds: 10.0,   // Check van collisions over last 10s
            van_collision_count_threshold: 3,     // Collided with van 3+ times
        }
    }
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct PlayerInputDirection(pub Vec2); // Placeholder for actual player input/intended direction

#[derive(Event, Debug)]
pub struct VanCollisionEvent {
    // Placeholder for collision events with van boundaries
    pub player_entity: Entity,
}

#[derive(Resource, Default)]
pub struct ErraticMovementTrackers {
    pub last_recorded_pos_for_displacement: Option<Vec2>,
    pub pos_record_time: f32,
    pub recent_direction_changes: std::collections::VecDeque<(f32, Vec2)>, // (timestamp, direction)
    pub recent_van_collisions: std::collections::VecDeque<f32>,            // (timestamp)
    pub last_known_direction: Option<Vec2>,
}

/// System to detect erratic player movement early in the mission.
pub fn erratic_movement_early_system(
    mut walkie_play: ResMut<WalkiePlay>,
    time: Res<Time>,
    level_ready_timer: Option<Res<LevelReadyTimer>>,
    game_state: Option<Res<GameState>>,
    config: Res<ErraticMovementEarlyConfig>,
    mut trackers: ResMut<ErraticMovementTrackers>,
    player_query: Query<(&Transform, &PlayerInputDirection), With<PlayerSprite>>,
    mut van_collision_reader: EventReader<VanCollisionEvent>, // Placeholder
) {
    let current_time_seconds = time.elapsed().as_secs_f32();

    let current_game_state = match game_state {
        Some(gs) => gs,
        None => return,
    };
    if *current_game_state != GameState::None {
        return;
    }

    let level_timer = match level_ready_timer {
        Some(lrt) => lrt,
        None => return,
    };

    if level_timer.paused() || level_timer.elapsed_secs() > config.time_threshold_early_seconds {
        // System active only during the early phase of the mission
        return;
    }

    let (player_transform, player_input_direction) = match player_query.get_single() {
        Ok(p) => p,
        Err(_) => return, // No player or multiple players
    };
    let current_player_pos = player_transform.translation.truncate();
    let current_player_dir = player_input_direction.0.normalize_or_zero();

    // Update trackers
    // Displacement tracking
    if trackers.last_recorded_pos_for_displacement.is_none()
        || current_time_seconds - trackers.pos_record_time > config.direction_change_window_seconds
    {
        trackers.last_recorded_pos_for_displacement = Some(current_player_pos);
        trackers.pos_record_time = current_time_seconds;
    }

    // Direction change tracking
    if current_player_dir != Vec2::ZERO {
        if let Some(last_dir) = trackers.last_known_direction {
            // Only record significant changes (e.g., > 30 degrees)
            if last_dir.angle_to(current_player_dir).abs() > 30.0f32.to_radians() {
                trackers
                    .recent_direction_changes
                    .push_back((current_time_seconds, current_player_dir));
            }
        } else {
            trackers
                .recent_direction_changes
                .push_back((current_time_seconds, current_player_dir));
        }
        trackers.last_known_direction = Some(current_player_dir);
    }
    while let Some((ts, _)) = trackers.recent_direction_changes.front() {
        if current_time_seconds - ts > config.direction_change_window_seconds {
            trackers.recent_direction_changes.pop_front();
        } else {
            break;
        }
    }

    // Van collision tracking
    for _event in van_collision_reader.read() {
        // Assuming the event correctly identifies the player entity, though not strictly needed here
        trackers
            .recent_van_collisions
            .push_back(current_time_seconds);
    }
    while let Some(ts) = trackers.recent_van_collisions.front() {
        if current_time_seconds - ts > config.van_collision_window_seconds {
            trackers.recent_van_collisions.pop_front();
        } else {
            break;
        }
    }

    // Check for erratic movement conditions
    let mut trigger_event = false;

    // Condition 1: High direction changes with low displacement
    if trackers.recent_direction_changes.len() >= config.direction_change_count_threshold {
        if let Some(start_pos) = trackers.last_recorded_pos_for_displacement {
            if current_player_pos.distance(start_pos) < config.low_displacement_threshold {
                trigger_event = true;
            }
        }
    }

    // Condition 2: Frequent van collisions
    if trackers.recent_van_collisions.len() >= config.van_collision_count_threshold {
        trigger_event = true;
    }

    if trigger_event {
        walkie_play.set(
            WalkieEvent::ErraticMovementEarly,
            time.elapsed().as_secs_f64(),
        );
    }
}

// --- Configuration and Timer for DoorInteractionHesitation ---
#[derive(Resource)]
pub struct DoorInteractionHesitationConfig {
    pub hesitation_threshold_seconds: f32,
    pub interaction_range: f32,
}

impl Default for DoorInteractionHesitationConfig {
    fn default() -> Self {
        Self {
            hesitation_threshold_seconds: 12.0, // Default: 12 seconds
            interaction_range: 2.0,             // Default: 2 units distance to door
        }
    }
}

#[derive(Component)]
pub struct MainEntranceDoor; // Tag component for the main entrance door entity

#[derive(Component, Default)] // Placeholder for door state
pub enum TileState {
    #[default]
    Open,
    Closed,
    Locked,
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct DoorHesitationTimer(pub Timer); // Specific timer for this scenario

/// System to detect if the player hesitates to interact with the main entrance door.
pub fn door_interaction_hesitation_system(
    mut walkie_play: ResMut<WalkiePlay>,
    time: Res<Time>,
    game_state: Option<Res<GameState>>,
    config: Res<DoorInteractionHesitationConfig>,
    mut door_hesitation_timer: ResMut<DoorHesitationTimer>,
    player_query: Query<&Transform, (With<PlayerSprite>, Without<MainEntranceDoor>)>,
    door_query: Query<(&Transform, &TileState), With<MainEntranceDoor>>, // Assuming door has a TileState
) {
    let current_game_state = match game_state {
        Some(gs) => gs,
        None => return,
    };
    if *current_game_state != GameState::None {
        door_hesitation_timer.pause();
        door_hesitation_timer.reset();
        return;
    }

    let player_transform = match player_query.get_single() {
        Ok(p_transform) => p_transform,
        Err(_) => return, // No player or multiple players
    };
    let player_position = player_transform.translation.truncate();

    let (door_transform, door_state) = match door_query.get_single() {
        Ok(d_data) => d_data,
        Err(_) => return, // No main entrance door found or multiple
    };
    let door_position = door_transform.translation.truncate();

    if player_position.distance(door_position) <= config.interaction_range
        && matches!(door_state, TileState::Closed)
    {
        // Player is near the closed door, start or continue the timer
        if door_hesitation_timer.paused() {
            door_hesitation_timer.unpause();
        }
        door_hesitation_timer.tick(time.delta());

        if door_hesitation_timer.elapsed_secs() > config.hesitation_threshold_seconds {
            walkie_play.set(
                WalkieEvent::DoorInteractionHesitation,
                time.elapsed().as_secs_f64(),
            );
            door_hesitation_timer.reset(); // Reset after triggering to avoid immediate re-trigger
            door_hesitation_timer.pause(); // And pause until conditions are met again
        }
    } else {
        // Player moved away or door is not closed/relevant, reset and pause timer
        door_hesitation_timer.reset();
        door_hesitation_timer.pause();
    }
}

// --- Configuration and Tracker for BumpingInDarkness ---
#[derive(Resource)]
pub struct BumpingInDarknessConfig {
    pub dark_threshold_lux: f32,
    pub collision_time_window_seconds: f32,
    pub collision_count_threshold: usize,
}

impl Default for BumpingInDarknessConfig {
    fn default() -> Self {
        Self {
            dark_threshold_lux: 0.1,
            collision_time_window_seconds: 7.0, // Check collisions over the last 7 seconds
            collision_count_threshold: 3,       // 3+ collisions with static objects
        }
    }
}

#[derive(Component)] // Placeholder for player's gear management
#[derive(Default)]
pub struct PlayerGear {
    pub flashlight_on: bool,
    // pub left_hand: Option<GearKind>, // More detailed approach
    // pub right_hand: Option<GearKind>, // More detailed approach
}

// Placeholder for overall board/level data, including light levels
#[derive(Resource, Default)]
pub struct BoardData {
    // Simplified: A function to get light level at a position.
    // In reality, this would query a light grid or similar structure.
    pub light_levels: std::collections::HashMap<IVec2, f32>, // Example: grid cell -> lux
}
impl BoardData {
    pub fn get_lux_at_position(&self, _player_pos: Vec2) -> f32 {
        // Placeholder logic: Assume a default low light unless specified in a hashmap by tile
        // let grid_pos = IVec2::new(player_pos.x.round() as i32, player_pos.y.round() as i32);
        // self.light_levels.get(&grid_pos).copied().unwrap_or(0.05) // Default very dark
        0.05 // Simplified: always return very dark for placeholder
    }
}

#[derive(Component, Resource, Default)] // Added Resource here
pub struct RoomDB {
    pub lights_on: bool,
}
// Removed conflicting manual impl Default for RoomDB

#[derive(Event, Debug)] // Placeholder for collision events with static environment
pub struct StaticCollisionEvent {
    pub player_entity: Entity,
    pub static_object_entity: Entity,
}

#[derive(Resource, Default)]
pub struct BumpingInDarknessCollisionTracker {
    pub recent_static_collisions: std::collections::VecDeque<f32>, // (timestamp)
}

/// System to detect if the player is bumping into things in the dark.
pub fn bumping_in_darkness_system(
    mut walkie_play: ResMut<WalkiePlay>,
    time: Res<Time>,
    game_state: Option<Res<GameState>>,
    config: Res<BumpingInDarknessConfig>,
    board_data: Option<Res<BoardData>>,
    mut collision_tracker: ResMut<BumpingInDarknessCollisionTracker>,
    player_query: Query<(&Transform, Option<&PlayerGear>), With<PlayerSprite>>,
    // Assuming current room's light status can be queried, e.g., via a resource or query on a Room entity
    // For simplicity, let's assume a single RoomDB resource for the current room, or that we'd query it.
    current_room_data: Option<Res<RoomDB>>, // Placeholder for current room's light status
    mut static_collision_reader: EventReader<StaticCollisionEvent>,
) {
    let current_time_seconds = time.elapsed().as_secs_f32();

    let current_game_state = match game_state {
        Some(gs) => gs,
        None => return,
    };
    if *current_game_state != GameState::None {
        return;
    }

    let (player_transform, player_gear) = match player_query.get_single() {
        Ok(p_data) => p_data,
        Err(_) => return, // No player or multiple players
    };
    let player_position = player_transform.translation.truncate();

    let light_at_player_pos = board_data.map_or(config.dark_threshold_lux, |bd| {
        bd.get_lux_at_position(player_position)
    });

    if light_at_player_pos >= config.dark_threshold_lux {
        collision_tracker.recent_static_collisions.clear(); // Clear history if it's not dark
        return; // Not dark enough to trigger this
    }

    let flashlight_on = player_gear.is_some_and(|gear| gear.flashlight_on);
    if flashlight_on {
        collision_tracker.recent_static_collisions.clear(); // Clear history if flashlight is on
        return; // Player is using flashlight
    }

    // Check room lights (placeholder logic)
    let room_lights_on = current_room_data.is_some_and(|room| room.lights_on);
    if room_lights_on {
        collision_tracker.recent_static_collisions.clear(); // Clear history if room lights are on
        return; // Room lights are on
    }

    // Track collisions
    for _event in static_collision_reader.read() {
        collision_tracker
            .recent_static_collisions
            .push_back(current_time_seconds);
    }
    while let Some(ts) = collision_tracker.recent_static_collisions.front() {
        if current_time_seconds - ts > config.collision_time_window_seconds {
            collision_tracker.recent_static_collisions.pop_front();
        } else {
            break;
        }
    }

    if collision_tracker.recent_static_collisions.len() >= config.collision_count_threshold {
        walkie_play.set(WalkieEvent::BumpingInDarkness, time.elapsed().as_secs_f64());
        collision_tracker.recent_static_collisions.clear(); // Clear after triggering
    }
}

// --- Configuration for StrugglingWithGrabDrop ---
#[derive(Resource)]
pub struct StrugglingWithGrabDropConfig {
    pub near_pickable_time_threshold_seconds: f32,
    pub pickable_interaction_range: f32,
    pub failed_action_collision_threshold: usize,
    // pub required_tutorial_chapter: u32, // Example: Only active from chapter 2 onwards
}

impl Default for StrugglingWithGrabDropConfig {
    fn default() -> Self {
        Self {
            near_pickable_time_threshold_seconds: 8.0,
            pickable_interaction_range: 1.5,
            failed_action_collision_threshold: 2,
            // required_tutorial_chapter: 2,
        }
    }
}

#[derive(Component)] // Placeholder for pickable items
pub struct PickableObject {
    pub weight: f32,
    pub name: String,
}

#[derive(Component)] // Placeholder for player's view cone / interaction target
pub struct PlayerViewConeTarget(pub Option<Entity>);

#[derive(Event, Debug)] // Placeholder for player input events
pub enum PlayerActionEvent {
    ActivateGear,
    CycleGear,
}

// Extending PlayerGear for held_item
#[derive(Component)]
pub struct HeldItem(pub Entity); // Entity of the item being held

// Timer for being near a pickable item without interaction
#[derive(Resource, Default, Deref, DerefMut)]
pub struct NearPickableTimer(pub Timer);

// Tracker for failed actions while holding an item
#[derive(Resource, Default)]
pub struct GrabDropActionTracker {
    pub failed_gear_activations_while_holding: u32,
    pub collisions_while_holding: u32,
    pub last_held_item_check_time: f32,
}

/// System to detect if the player is struggling with grab/drop mechanics.
pub fn struggling_with_grab_drop_system(
    mut walkie_play: ResMut<WalkiePlay>,
    time: Res<Time>,
    game_state: Option<Res<GameState>>,
    config: Res<StrugglingWithGrabDropConfig>,
    // tutorial_progress: Option<Res<TutorialProgress>>, // Placeholder for tutorial state
    mut near_pickable_timer: ResMut<NearPickableTimer>,
    mut action_tracker: ResMut<GrabDropActionTracker>,
    player_query: Query<
        (
            Entity,
            &Transform,
            &PlayerGear, // Assumed to be extended or separate for held_item
            Option<&HeldItem>,
            Option<&PlayerViewConeTarget>,
        ),
        With<PlayerSprite>,
    >,
    pickable_query: Query<(Entity, &Transform, &PickableObject), Without<PlayerSprite>>,
    mut player_action_reader: EventReader<PlayerActionEvent>, // For [R]/[Tab]
    mut static_collision_reader: EventReader<StaticCollisionEvent>, // Re-use for collisions
) {
    let current_time_seconds = time.elapsed().as_secs_f32();

    let current_game_state = match game_state {
        Some(gs) => gs,
        None => return,
    };
    if *current_game_state != GameState::None {
        near_pickable_timer.pause();
        near_pickable_timer.reset();
        return;
    }

    // // Placeholder: Check tutorial progression
    // if let Some(progress) = tutorial_progress {
    //     if progress.current_chapter < config.required_tutorial_chapter {
    //         return;
    //     }
    // }

    let (_player_entity, player_transform, _player_gear, held_item, view_target) =
        match player_query.get_single() {
            Ok(p_data) => p_data,
            Err(_) => return,
        };
    let player_position = player_transform.translation.truncate();

    let mut trigger_event = false;

    if held_item.is_none() {
        // --- Scenario: Not Picking Up ---
        let mut near_unheld_pickable = false;
        for (pickable_entity, pickable_transform, _pickable_object) in pickable_query.iter() {
            if player_position.distance(pickable_transform.translation.truncate())
                < config.pickable_interaction_range
            {
                // Optional: Check if player is looking at it (view_target)
                if let Some(target) = view_target {
                    if target.0 == Some(pickable_entity) {
                        near_unheld_pickable = true;
                        break;
                    }
                } else {
                    // Simpler check if no view cone
                    near_unheld_pickable = true;
                    break;
                }
            }
        }

        if near_unheld_pickable {
            if near_pickable_timer.paused() {
                near_pickable_timer.unpause();
            }
            near_pickable_timer.tick(time.delta());
            if near_pickable_timer.elapsed_secs() > config.near_pickable_time_threshold_seconds {
                trigger_event = true;
            }
        } else {
            near_pickable_timer.reset();
            near_pickable_timer.pause();
        }
    } else {
        // --- Scenario: Holding and Failing Action ---
        near_pickable_timer.reset(); // Not relevant if holding something
        near_pickable_timer.pause();

        // Reset tracker if not checked recently
        if current_time_seconds - action_tracker.last_held_item_check_time > 5.0 {
            // Reset counts every 5s
            action_tracker.failed_gear_activations_while_holding = 0;
            action_tracker.collisions_while_holding = 0;
        }
        action_tracker.last_held_item_check_time = current_time_seconds;

        for action_event in player_action_reader.read() {
            match action_event {
                PlayerActionEvent::ActivateGear | PlayerActionEvent::CycleGear => {
                    action_tracker.failed_gear_activations_while_holding += 1;
                }
            }
        }
        for _collision_event in static_collision_reader.read() {
            action_tracker.collisions_while_holding += 1;
        }

        if action_tracker.failed_gear_activations_while_holding > 0
            || action_tracker.collisions_while_holding
                >= config.failed_action_collision_threshold as u32
        {
            trigger_event = true;
        }
    }

    if trigger_event {
        walkie_play.set(
            WalkieEvent::StrugglingWithGrabDrop,
            time.elapsed().as_secs_f64(),
        );
        // Reset timers/trackers to avoid immediate re-trigger for the same scenario
        near_pickable_timer.reset();
        near_pickable_timer.pause();
        action_tracker.failed_gear_activations_while_holding = 0;
        action_tracker.collisions_while_holding = 0;
    }
}

// --- Configuration for StrugglingWithHideUnhide ---
#[derive(Resource)]
pub struct StrugglingWithHideUnhideConfig {
    pub near_hiding_spot_no_hide_time_seconds: f32,
    pub immediate_unhide_time_seconds: f32,
    pub hiding_spot_interaction_range: f32,
    // pub required_tutorial_chapter: u32, // Example: Only active from chapter 2
}

impl Default for StrugglingWithHideUnhideConfig {
    fn default() -> Self {
        Self {
            near_hiding_spot_no_hide_time_seconds: 3.0, // Short time for urgent situations
            immediate_unhide_time_seconds: 2.0,         // Very short for quick unhide
            hiding_spot_interaction_range: 1.8,
            // required_tutorial_chapter: 2,
        }
    }
}

#[derive(Component)] // Placeholder for ghost state
pub struct GhostSprite {
    pub hunting: f32,
    /* > 0 if hunting */ pub rage: f32,
}
impl Default for GhostSprite {
    fn default() -> Self {
        Self {
            hunting: 0.0,
            rage: 0.0,
        }
    }
}

#[derive(Component)] // Placeholder for hiding spots
pub struct HidingSpot;

#[derive(Component)] // Placeholder for player being hidden
pub struct Hiding;

#[derive(Event, Debug)] // Placeholder for player input for interaction
pub struct PlayerInteractEvent {
    pub target: Option<Entity>,
    pub action_type: InteractionType,
}
#[derive(Debug)] // Added Debug derive
pub enum InteractionType {
    Primary,
    Secondary,
    HideKeyPress,
    HideKeyRelease,
}

#[derive(Resource, Default)]
pub struct HideUnhideTracker {
    pub time_entered_hiding_spot: Option<f32>,
    pub time_near_hiding_spot_while_hunted: Option<f32>,
}

/// System to detect if the player is struggling with hide/unhide mechanics.
pub fn struggling_with_hide_unhide_system(
    mut walkie_play: ResMut<WalkiePlay>,
    time: Res<Time>,
    game_state: Option<Res<GameState>>,
    config: Res<StrugglingWithHideUnhideConfig>,
    // tutorial_progress: Option<Res<TutorialProgress>>,
    mut tracker: ResMut<HideUnhideTracker>,
    player_query: Query<
        (
            Entity,
            &Transform,
            &PlayerGear, // For held_item check
            Option<&HeldItem>,
            Option<&Hiding>,
        ),
        With<PlayerSprite>,
    >,
    ghost_query: Query<&GhostSprite>, // Assuming one ghost
    hiding_spot_query: Query<(Entity, &Transform), With<HidingSpot>>,
    mut interaction_reader: EventReader<PlayerInteractEvent>,
) {
    let current_time_seconds = time.elapsed().as_secs_f32();

    let current_game_state = match game_state {
        Some(gs) => gs,
        None => return,
    };
    if *current_game_state != GameState::None {
        return;
    }

    // // Placeholder: Check tutorial progression
    // if let Some(progress) = tutorial_progress {
    //     if progress.current_chapter < config.required_tutorial_chapter {
    //         return;
    //     }
    // }

    let (_player_entity, player_transform, _player_gear, held_item, hiding_status) =
        match player_query.get_single() {
            Ok(p_data) => p_data,
            Err(_) => return,
        };
    let player_position = player_transform.translation.truncate();

    let ghost_sprite = match ghost_query.get_single() {
        Ok(g) => g,
        Err(_) => return, // No ghost or multiple ghosts
    };
    let is_hunting = ghost_sprite.hunting > 0.0;

    let mut trigger_event = false;
    let event_to_send = WalkieEvent::StrugglingWithHideUnhide; // Default, can be more specific

    // --- Scenario: Not Hiding During Hunt Near Spot ---
    if is_hunting && hiding_status.is_none() {
        let mut near_available_hiding_spot = false;
        for (_spot_entity, spot_transform) in hiding_spot_query.iter() {
            if player_position.distance(spot_transform.translation.truncate())
                < config.hiding_spot_interaction_range
            {
                near_available_hiding_spot = true;
                break;
            }
        }

        if near_available_hiding_spot {
            if tracker.time_near_hiding_spot_while_hunted.is_none() {
                tracker.time_near_hiding_spot_while_hunted = Some(current_time_seconds);
            }
            if let Some(time_near) = tracker.time_near_hiding_spot_while_hunted {
                if current_time_seconds - time_near > config.near_hiding_spot_no_hide_time_seconds {
                    trigger_event = true;
                }
            }
        } else {
            tracker.time_near_hiding_spot_while_hunted = None; // Reset if player moves away
        }
    } else {
        tracker.time_near_hiding_spot_while_hunted = None; // Reset if not hunting or already hiding
    }

    // --- Scenario: Immediately Unhiding ---
    if is_hunting && hiding_status.is_some() {
        // This requires knowing when Hiding component was added. Assume it's recent.
        // A more robust way: listen for Hiding component removal event.
        // For now, using time_entered_hiding_spot from tracker.
        // if let Some(_time_hidden) = tracker.time_entered_hiding_spot {
        // This part is tricky: if Hiding is removed, this system might not catch it immediately.
        // This logic is more for *detecting* a quick unhide if the Hiding component is still there
        // but about to be removed by player input in the same frame or very soon.
        // A better approach would be to react to an UnhideEvent.
        // }
    }
    // If Hiding component was just added, record time
    if hiding_status.is_some() && tracker.time_entered_hiding_spot.is_none() {
        tracker.time_entered_hiding_spot = Some(current_time_seconds);
    }
    // If Hiding component is gone, check if it was a quick unhide
    if hiding_status.is_none() && tracker.time_entered_hiding_spot.is_some() {
        if let Some(time_hidden) = tracker.time_entered_hiding_spot {
            if is_hunting
                && (current_time_seconds - time_hidden < config.immediate_unhide_time_seconds)
            {
                trigger_event = true;
            }
        }
        tracker.time_entered_hiding_spot = None; // Reset once unhidden
    }

    // --- Scenario: Trying to Hide While Carrying ---
    for interact_event in interaction_reader.read() {
        if matches!(interact_event.action_type, InteractionType::HideKeyPress)
            && held_item.is_some()
        {
            if let Some(target_entity) = interact_event.target {
                if hiding_spot_query.get(target_entity).is_ok() {
                    // Check if target is a hiding spot
                    trigger_event = true;
                    break;
                }
            }
        }
    }

    if trigger_event {
        walkie_play.set(event_to_send, time.elapsed().as_secs_f64());
        // Reset relevant parts of tracker to avoid re-trigger for the same immediate scenario
        tracker.time_near_hiding_spot_while_hunted = None;
        // tracker.time_entered_hiding_spot = None; // Don't reset this one too eagerly
    }
}

// TODO: Implement trigger systems for StrugglingWithGrabDrop, etc.
// TODO: System to start/reset LevelReadyTimer on a specific event (e.g., LevelReadyEvent)
// pub fn handle_level_ready_event(
//     mut event_reader: EventReader<LevelReadyEvent>, // Assuming such an event exists
//     mut level_ready_timer: ResMut<LevelReadyTimer>,
// ) {
//     if !event_reader.is_empty() {
//         event_reader.clear();
//         level_ready_timer.0 = Timer::from_seconds(f32::MAX, TimerMode::Once); // Effectively a stopwatch
//         // OR if it's a countdown for a specific initial period:
//         // level_ready_timer.0 = Timer::from_seconds(INITIAL_GRACE_PERIOD, TimerMode::Once);
//         info!("LevelReadyEvent detected, (re)starting LevelReadyTimer for walkie triggers.");
//     }
// }
