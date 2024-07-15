//! Player Module
//! -------------
//!
//! This module defines the player character and its interactions with the game world, including:
//! * The `PlayerSprite` component, which stores player attributes, controls, and state.
//! * Systems for handling player input, movement, collisions, interactions with objects, and sanity changes.
//! * Data structures and enums for managing player controls, animations, and held objects.

use crate::behavior::component::{Interactive, RoomState};
use crate::behavior::{self, Behavior};
use crate::board::{self, Bdl, BoardData, BoardPosition, Position};
use crate::difficulty::CurrentDifficulty;
use crate::game;
use crate::game::level::{InteractionExecutionType, RoomChangedEvent};
use crate::game::{ui::DamageBackground, GameConfig};
use crate::gear::playergear::PlayerGear;
use crate::gear::{self, GearUsable as _};
use crate::maplight::MapColor;
use crate::npchelp::NpcHelpEvent;
use crate::{maplight, root, utils};
use bevy::color::palettes::css;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::utils::HashMap;
use std::time::Duration;

const USE_ARROW_KEYS: bool = false;

/// Represents a piece of gear deployed in the game world.
#[derive(Component, Debug, Clone)]
pub struct DeployedGear {
    /// The direction the gear is facing.
    pub direction: board::Direction,
}

/// Component to store the GearKind of a deployed gear entity.
#[derive(Component, Debug, Clone)]
pub struct DeployedGearData {
    pub gear: gear::Gear,
}

/// Enables/disables debug logs related to the player.
const DEBUG_PLAYER: bool = false;

/// Represents a player character in the game world.
///
/// This component stores the player's attributes, control scheme, sanity level, health, and mean sound exposure.
#[derive(Component, Debug)]
pub struct PlayerSprite {
    /// The unique identifier for the player (e.g., Player 1, Player 2).
    pub id: usize,
    /// The keyboard control scheme for the player (WASD, IJKL, etc.).
    pub controls: ControlKeys,
    /// The player's accumulated "craziness" level. Higher craziness reduces sanity.
    pub crazyness: f32,
    /// The average sound level the player has been exposed to, used for sanity calculations.
    pub mean_sound: f32,
    /// The player's current health. A value of 0 indicates the player is incapacitated.
    pub health: f32,
}

impl PlayerSprite {
    /// Creates a new `PlayerSprite` with the specified ID and default controls.
    pub fn new(id: usize) -> Self {
        Self {
            id,
            controls: Self::default_controls(id),
            crazyness: 0.0,
            mean_sound: 0.0,
            health: 100.0,
        }
    }

    /// Returns a modified version with the requested sanity
    pub fn with_sanity(self, sanity: f32) -> Self {
        Self {
            crazyness: Self::required_crazyness(sanity),
            ..self
        }
    }

    /// Calculates the required crazyness based on the player's current sanity level.
    pub fn required_crazyness(sanity: f32) -> f32 {
        const LINEAR: f32 = 30.0;
        const SCALE: f32 = 100.0;
        (SCALE * LINEAR).powi(2) / (sanity * sanity) - LINEAR.powi(2)
    }

    /// Returns the default `ControlKeys` for the given player ID.
    fn default_controls(id: usize) -> ControlKeys {
        match id {
            1 => {
                if USE_ARROW_KEYS {
                    ControlKeys::ARROWS
                } else {
                    ControlKeys::WASD
                }
            }
            2 => ControlKeys::IJKL,
            _ => ControlKeys::NONE,
        }
    }

    /// Calculates the player's current sanity level based on their accumulated craziness.
    pub fn sanity(&self) -> f32 {
        const LINEAR: f32 = 30.0;
        const SCALE: f32 = 100.0;
        (SCALE * LINEAR) / ((self.crazyness + LINEAR * LINEAR).sqrt())
    }
}

/// Defines the keyboard controls for a player.
#[derive(Debug, Clone)]
pub struct ControlKeys {
    /// Key for moving up.
    pub up: KeyCode,
    /// Key for moving down.
    pub down: KeyCode,
    /// Key for moving left.
    pub left: KeyCode,
    /// Key for moving right.
    pub right: KeyCode,

    /// Key for interacting with objects (doors, switches, etc.).
    pub activate: KeyCode,
    /// Key for grabbing objects.
    pub grab: KeyCode,
    /// Key for dropping objects.
    pub drop: KeyCode,
    /// Key for triggering the left-hand item (e.g., flashlight).
    pub torch: KeyCode,
    /// Key for triggering the right-hand item (e.g., EMF reader).
    pub trigger: KeyCode,
    /// Key for cycling through inventory items.
    pub cycle: KeyCode,
    /// Key for swapping left and right hand items.
    pub swap: KeyCode,
    /// Key for changing the evidence selection in the quick menu.
    pub change_evidence: KeyCode,
}

/// System for handling player movement, interaction, and collision.
///
/// This system processes player input, updates the player's position and direction,
/// handles interactions with interactive objects, and manages collisions with the environment.
impl ControlKeys {
    pub const WASD: Self = ControlKeys {
        up: KeyCode::KeyW,
        down: KeyCode::KeyS,
        left: KeyCode::KeyA,
        right: KeyCode::KeyD,
        activate: KeyCode::KeyE,
        trigger: KeyCode::KeyR,
        torch: KeyCode::Tab,
        cycle: KeyCode::KeyQ,
        swap: KeyCode::KeyT,
        drop: KeyCode::KeyG,
        grab: KeyCode::KeyF,
        change_evidence: KeyCode::KeyC,
    };
    pub const ARROWS: Self = ControlKeys {
        up: KeyCode::ArrowUp,
        down: KeyCode::ArrowDown,
        left: KeyCode::ArrowLeft,
        right: KeyCode::ArrowRight,
        activate: KeyCode::KeyE,
        trigger: KeyCode::KeyR,
        torch: KeyCode::Tab,
        cycle: KeyCode::KeyQ,
        swap: KeyCode::KeyT,
        drop: KeyCode::KeyG,
        grab: KeyCode::KeyF,
        change_evidence: KeyCode::KeyC,
    };
    pub const IJKL: Self = ControlKeys {
        up: KeyCode::KeyI,
        down: KeyCode::KeyK,
        left: KeyCode::KeyJ,
        right: KeyCode::KeyL,
        activate: KeyCode::KeyO,
        torch: KeyCode::KeyT,
        cycle: KeyCode::NonConvert,
        swap: KeyCode::NonConvert,
        grab: KeyCode::NonConvert,
        drop: KeyCode::NonConvert,
        trigger: KeyCode::NonConvert,
        change_evidence: KeyCode::NonConvert,
    };
    pub const NONE: Self = ControlKeys {
        up: KeyCode::NonConvert,
        down: KeyCode::NonConvert,
        left: KeyCode::NonConvert,
        right: KeyCode::NonConvert,
        activate: KeyCode::NonConvert,
        torch: KeyCode::NonConvert,
        cycle: KeyCode::NonConvert,
        swap: KeyCode::NonConvert,
        grab: KeyCode::NonConvert,
        drop: KeyCode::NonConvert,
        trigger: KeyCode::NonConvert,
        change_evidence: KeyCode::NonConvert,
    };
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn keyboard_player(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(
        &mut board::Position,
        &mut board::Direction,
        &mut PlayerSprite,
        &mut AnimationTimer,
        &PlayerGear,
        Option<&Hiding>,
    )>,
    colhand: CollisionHandler,
    interactables: Query<
        (
            Entity,
            &board::Position,
            &Interactive,
            &Behavior,
            Option<&RoomState>,
        ),
        Without<PlayerSprite>,
    >,
    mut interactive_stuff: InteractiveStuff,
    mut ev_room: EventWriter<RoomChangedEvent>,
    mut ev_npc: EventWriter<NpcHelpEvent>,
    difficulty: Res<CurrentDifficulty>, // Access the difficulty settings
) {
    const PLAYER_SPEED: f32 = 0.04;
    const DIR_MIN: f32 = 5.0;
    const DIR_MAX: f32 = 80.0;
    const DIR_STEPS: f32 = 15.0;
    const DIR_MAG2: f32 = DIR_MAX / DIR_STEPS;
    const DIR_RED: f32 = 1.001;
    let dt = time.delta_seconds() * 60.0;
    for (mut pos, mut dir, player, mut anim, player_gear, hiding) in players.iter_mut() {
        let col_delta = colhand.delta(&pos);
        pos.x -= col_delta.x;
        pos.y -= col_delta.y;

        let mut d = board::Direction {
            dx: 0.0,
            dy: 0.0,
            dz: 0.0,
        };

        if keyboard_input.pressed(player.controls.up) {
            d.dy += 1.0;
        }
        if keyboard_input.pressed(player.controls.down) {
            d.dy -= 1.0;
        }
        if keyboard_input.pressed(player.controls.left) {
            d.dx -= 1.0;
        }
        if keyboard_input.pressed(player.controls.right) {
            d.dx += 1.0;
        }

        d = d.normalized();
        let col_delta_n = (col_delta * 100.0).clamp_length_max(1.0);
        let col_dotp = (d.dx * col_delta_n.x + d.dy * col_delta_n.y).clamp(0.0, 1.0);
        d.dx -= col_delta_n.x * col_dotp;
        d.dy -= col_delta_n.y * col_dotp;

        let delta = d / 0.1 + dir.normalized() / DIR_MAG2 / 1000.0;

        // d.dx /= 1.5; // Compensate for the projection

        // --- Speed Penalty Based on Held Object Weight ---
        let speed_penalty = if player_gear.held_item.is_some() {
            0.5
        } else {
            1.0
        };
        dir.dx += DIR_MAG2 * d.dx;
        dir.dy += DIR_MAG2 * d.dy;

        let dir_dist = (dir.dx.powi(2) + dir.dy.powi(2)).sqrt();
        if dir_dist > DIR_MAX {
            dir.dx *= DIR_MAX / dir_dist;
            dir.dy *= DIR_MAX / dir_dist;
        } else if dir_dist > DIR_MIN {
            dir.dx /= DIR_RED;
            dir.dy /= DIR_RED;
        }

        // --- Check if Player is Hiding ---
        if hiding.is_some() {
            // Update player animation
            let dscreen = delta.to_screen_coord();
            anim.set_range(
                CharacterAnimation::from_dir(dscreen.x / 2000.0, dscreen.y / 1000.0).to_vec(),
            );

            // Check if the Hiding component is present
            continue; // Skip movement input handling if hiding
        }

        // Apply speed penalty
        pos.x += PLAYER_SPEED * d.dx * dt * speed_penalty * difficulty.0.player_speed;
        pos.y += PLAYER_SPEED * d.dy * dt * speed_penalty * difficulty.0.player_speed;

        // Update player animation
        let dscreen = delta.to_screen_coord();
        anim.set_range(CharacterAnimation::from_dir(dscreen.x, dscreen.y * 2.0).to_vec());

        // ----
        if keyboard_input.just_pressed(player.controls.activate) {
            // let d = dir.normalized();
            let mut max_dist = 1.4;
            let mut selected_entity = None;
            for (entity, item_pos, interactive, behavior, _) in interactables.iter() {
                let cp_delta = interactive.control_point_delta(behavior);
                // let old_dist = pos.delta(*item_pos);
                let item_pos = Position {
                    x: item_pos.x + cp_delta.x,
                    y: item_pos.y + cp_delta.y,
                    z: item_pos.z + cp_delta.z,
                    global_z: item_pos.global_z,
                };
                let new_dist = pos.delta(item_pos);
                // let new_dist_norm = new_dist.normalized();
                // let dot_p = (new_dist_norm.dx * -d.dx + new_dist_norm.dy * -d.dy).clamp(0.0, 1.0);
                // let dref = new_dist + (&d * (new_dist.distance().min(1.0) * dot_p));
                let dref = new_dist;
                let dist = dref.distance();
                if dist < max_dist {
                    max_dist = dist + 0.00001;
                    selected_entity = Some(entity);
                }
            }
            if let Some(entity) = selected_entity {
                for (entity, item_pos, interactive, behavior, rs) in
                    interactables.iter().filter(|(e, _, _, _, _)| *e == entity)
                {
                    if behavior.is_npc() {
                        ev_npc.send(NpcHelpEvent::new(entity));
                    }

                    if interactive_stuff.execute_interaction(
                        entity,
                        item_pos,
                        Some(interactive),
                        behavior,
                        rs,
                        InteractionExecutionType::ChangeState,
                    ) {
                        ev_room.send(RoomChangedEvent::default());
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CharacterAnimationDirection {
    NN,
    NW,
    WW,
    SW,
    SS,
    SE,
    EE,
    NE,
}

impl CharacterAnimationDirection {
    fn from_dir(dx: f32, dy: f32) -> Self {
        let dst = (dx * dx + dy * dy).sqrt() + 0.0000000001;
        let dx = (dx / dst).round() as i32;
        let dy = (dy / dst).round() as i32;
        match dx {
            1 => match dy {
                1 => Self::NE,
                -1 => Self::SE,
                _ => Self::EE,
            },
            0 => match dy {
                1 => Self::NN,
                -1 => Self::SS,
                _ => Self::SS,
            },
            -1 => match dy {
                1 => Self::NW,
                -1 => Self::SW,
                _ => Self::WW,
            },
            _ => Self::EE,
        }
    }
    fn to_delta_idx(self) -> usize {
        match self {
            Self::NN => 0,
            Self::NW => 1,
            Self::WW => 2,
            Self::SW => 3,
            Self::SS => 16,
            Self::SE => 17,
            Self::EE => 18,
            Self::NE => 19,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CharacterAnimationState {
    Standing,
    Walking,
}

impl CharacterAnimationState {
    fn to_delta_idx(self) -> usize {
        match self {
            Self::Standing => 32,
            Self::Walking => 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CharacterAnimation {
    pub state: CharacterAnimationState,
    pub dir: CharacterAnimationDirection,
}

impl CharacterAnimation {
    pub fn from_dir(dx: f32, dy: f32) -> Self {
        let dst = (dx * dx + dy * dy).sqrt() + 0.001;
        let state = if dst > 1.0 {
            CharacterAnimationState::Walking
        } else {
            CharacterAnimationState::Standing
        };
        let dir = CharacterAnimationDirection::from_dir(dx, dy);
        Self { state, dir }
    }
    pub fn to_vec(self) -> Vec<usize> {
        let i = self.state.to_delta_idx() + self.dir.to_delta_idx();
        vec![i, i + 4, i + 8, i + 12]
    }
}

#[derive(Component)]
pub struct AnimationTimer {
    timer: Timer,
    // range: RangeInclusive<usize>,
    frames: Vec<usize>,
    idx: usize,
}

impl AnimationTimer {
    pub fn from_range<I: IntoIterator<Item = usize>>(timer: Timer, range: I) -> Self {
        let frames: Vec<usize> = range.into_iter().collect();
        AnimationTimer {
            timer,
            frames,
            idx: 0,
        }
    }
    pub fn set_range<I: IntoIterator<Item = usize>>(&mut self, range: I) {
        self.frames = range.into_iter().collect();
    }
    pub fn tick(&mut self, delta: Duration) -> Option<usize> {
        self.timer.tick(delta);
        if !self.timer.just_finished() {
            return None;
        }
        self.idx = (self.idx + 1) % self.frames.len();
        Some(self.frames[self.idx])
    }
}

pub fn animate_sprite(time: Res<Time>, mut query: Query<(&mut AnimationTimer, &mut TextureAtlas)>) {
    for (mut anim, mut texture_atlas) in query.iter_mut() {
        if let Some(idx) = anim.tick(time.delta()) {
            texture_atlas.index = idx;
        }
    }
}

/// System parameter for handling player collisions with the environment.
#[derive(SystemParam)]
pub struct CollisionHandler<'w> {
    /// Access to the game's board data, including collision information.
    bf: Res<'w, board::BoardData>,
}

impl<'w> CollisionHandler<'w> {
    const ENABLE_COLLISION: bool = true;
    const PILLAR_SZ: f32 = 0.3;
    const PLAYER_SZ: f32 = 0.5;

    fn delta(&self, pos: &Position) -> Vec3 {
        let bpos = pos.to_board_position();
        let mut delta = Vec3::ZERO;
        for npos in bpos.xy_neighbors(1) {
            let cf = self
                .bf
                .collision_field
                .get(&npos)
                .copied()
                .unwrap_or_default();
            if !cf.player_free && Self::ENABLE_COLLISION {
                let dpos = npos.to_position().to_vec3() - pos.to_vec3();
                let mut dapos = dpos.abs();
                dapos.x -= Self::PILLAR_SZ;
                dapos.y -= Self::PILLAR_SZ;
                dapos.x = dapos.x.max(0.0);
                dapos.y = dapos.y.max(0.0);
                let ddist = dapos.distance(Vec3::ZERO);
                if ddist < Self::PLAYER_SZ {
                    if dpos.x < 0.0 {
                        dapos.x *= -1.0;
                    }
                    if dpos.y < 0.0 {
                        dapos.y *= -1.0;
                    }
                    let fix_dist = (Self::PLAYER_SZ - ddist).powi(2);
                    let dpos_fix = dapos / (ddist + 0.000001) * fix_dist;
                    delta += dpos_fix;
                }
            }
        }
        delta
    }
}

/// The `InteractiveStuff` system handles interactions between the player and interactive objects
/// in the game world, such as doors, switches, lamps, and the van entry.
///
/// This system centralizes the logic for:
///  * Changing the state of interactive objects based on player interaction or room state.
///  * Playing appropriate sound effects for different interactions.
///  * Triggering transitions to the truck UI when the player enters the van.
///
#[derive(SystemParam)]
pub struct InteractiveStuff<'w, 's> {
    /// Database of sprites for map tiles. Used to retrieve alternative sprites for interactive objects.
    pub bf: Res<'w, board::SpriteDB>,
    /// Used to spawn sound effects and potentially other entities related to interactions.
    pub commands: Commands<'w, 's>,
    /// Access to the asset server for loading sound effects.
    pub asset_server: Res<'w, AssetServer>,
    /// Access to the materials used for rendering map tiles. Used to update tile visuals
    /// when object states change.
    pub materials1: ResMut<'w, Assets<crate::materials::CustomMaterial1>>,
    /// Database of room data, used to track the state of rooms and update interactive
    /// objects accordingly.
    pub roomdb: ResMut<'w, board::RoomDB>,
    /// Controls the transition to different game states, such as the truck UI.
    pub game_next_state: ResMut<'w, NextState<root::GameState>>,
}

impl<'w, 's> InteractiveStuff<'w, 's> {
    /// Executes an interaction with an interactive object.
    ///
    /// This method determines the object's new state based on the type of interaction, updates its `Behavior` component,
    /// plays the corresponding sound effect, and updates the room state if applicable.
    ///
    /// # Parameters:
    ///
    /// * `entity`: The entity of the interactive object.
    /// * `item_pos`: The position of the interactive object in the game world.
    /// * `interactive`: The `Interactive` component of the object, if present.
    /// * `behavior`: The `Behavior` component of the object.
    /// * `room_state`: The `RoomState` component of the object, if present.
    /// * `ietype`: The type of interaction being executed (`ChangeState` or `ReadRoomState`).
    ///
    /// # Returns:
    ///
    /// `true` if the interaction resulted in a change to the object's state, `false` otherwise.
    pub fn execute_interaction(
        &mut self,
        entity: Entity,
        item_pos: &Position,
        interactive: Option<&Interactive>,
        behavior: &Behavior,
        room_state: Option<&RoomState>,
        ietype: InteractionExecutionType,
    ) -> bool {
        let item_bpos = item_pos.to_board_position();
        let tuid = behavior.key_tuid();
        let cvo = behavior.key_cvo();
        if behavior.is_van_entry() {
            if ietype != InteractionExecutionType::ChangeState {
                return false;
            }
            if let Some(interactive) = interactive {
                let sound_file = interactive.sound_for_moving_into_state(behavior);
                self.commands.spawn(AudioBundle {
                    source: self.asset_server.load(sound_file),
                    settings: PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Despawn,
                        volume: bevy::audio::Volume::new(1.0),
                        speed: 1.0,
                        paused: false,
                        spatial: false,
                        spatial_scale: None,
                    },
                });
            }
            self.game_next_state.set(root::GameState::Truck);
            return false;
        }
        for other_tuid in self.bf.cvo_idx.get(&cvo).unwrap().iter() {
            if *other_tuid == tuid {
                continue;
            }
            let mut e_commands = self.commands.get_entity(entity).unwrap();
            let other = self.bf.map_tile.get(other_tuid).unwrap();

            let mut beh = other.behavior.clone();
            beh.flip(behavior.p.flip);

            // In case it is connected to a room, we need to change room state.
            if let Some(room_state) = room_state {
                let item_roombpos = BoardPosition {
                    x: item_bpos.x + room_state.room_delta.x,
                    y: item_bpos.y + room_state.room_delta.y,
                    z: item_bpos.z + room_state.room_delta.z,
                };
                let room_name = self
                    .roomdb
                    .room_tiles
                    .get(&item_roombpos)
                    .cloned()
                    .unwrap_or_default();
                // dbg!(&room_state, &item_roombpos);
                // dbg!(&room_name);
                match ietype {
                    InteractionExecutionType::ChangeState => {
                        if let Some(main_room_state) = self.roomdb.room_state.get_mut(&room_name) {
                            *main_room_state = beh.state();
                        }
                    }
                    InteractionExecutionType::ReadRoomState => {
                        if let Some(main_room_state) = self.roomdb.room_state.get(&room_name) {
                            if *main_room_state != beh.state() {
                                continue;
                            }
                        }
                    }
                }
            }

            match other.bundle.clone() {
                Bdl::Mmb(b) => {
                    let mat = self.materials1.get(&b.material).unwrap().clone();
                    let mat = self.materials1.add(mat);

                    e_commands.insert(mat);
                }
                Bdl::Sb(b) => {
                    e_commands.insert(b);
                }
            };

            e_commands.insert(beh);
            if ietype == InteractionExecutionType::ChangeState {
                if let Some(interactive) = interactive {
                    let sound_file = interactive.sound_for_moving_into_state(&other.behavior);
                    self.commands.spawn(AudioBundle {
                        source: self.asset_server.load(sound_file),
                        settings: PlaybackSettings {
                            mode: bevy::audio::PlaybackMode::Despawn,
                            volume: bevy::audio::Volume::new(1.0),
                            speed: 1.0,
                            paused: false,
                            spatial: false,
                            spatial_scale: None,
                        },
                    });
                }
            }

            return true;
        }
        false
    }
}

/// Represents an object that is currently being held by the player.
#[derive(Component, Debug, Clone)]
pub struct HeldObject {
    pub entity: Entity,
}

/// Marks a player entity that is currently hiding.
#[derive(Component)]
pub struct Hiding {
    pub hiding_spot: Entity,
}

/// Allows the player to pick up a pickable object from the environment.
///
/// This system checks if the player is pressing the 'grab' key and if there is a pickable object within reach.
/// If so, the object is visually attached to the player, and the player's right-hand gear is disabled.
/// Only one object can be held at a time.
pub fn grab_object(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&mut PlayerGear, &Position, &PlayerSprite)>,
    deployables: Query<(Entity, &Position), With<DeployedGear>>,
    pickables: Query<(Entity, &Position, &Behavior)>, // Query for all entities with Behavior
    mut gs: gear::GearStuff,
) {
    for (mut player_gear, player_pos, player) in players.iter_mut() {
        if keyboard_input.just_pressed(player.controls.grab) && player_gear.held_item.is_none() {
            // If there's any gear deployed nearby do not consider furniture.
            if deployables
                .iter()
                .any(|(_, object_pos)| player_pos.distance(object_pos) < 1.0)
            {
                return;
            }

            // Find a pickable object near the player
            if let Some((object_entity, _, _)) = pickables
                .iter()
                .filter(|(_, _, behavior)| behavior.p.object.pickable) // Filter for pickable objects
                .find(|(_, object_pos, _)| player_pos.distance(object_pos) < 1.0)
            {
                // Set the held object in the player's gear
                player_gear.held_item = Some(HeldObject {
                    entity: object_entity,
                });

                // Play "Pick Up" sound effect
                gs.play_audio("sounds/item-pickup-whoosh.ogg".into(), 1.0, player_pos);
            }
        }
    }
}

/// Allows the player to release a held object back into the environment.
///
/// This system checks if the player is pressing the 'drop' key and if they are currently holding an object.
/// It then determines if the target tile (the player's current position) is a valid drop location
/// (an empty floor tile and not obstructed by other objects).
///
/// If the drop is valid, the object is placed at the target tile.
/// If the drop is invalid, an "invalid drop" sound effect is played, and the object is not dropped.
#[allow(clippy::type_complexity)]
pub fn drop_object(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&mut PlayerGear, &Position, &PlayerSprite), Without<Behavior>>,
    mut objects: Query<
        (Entity, &mut Position),
        (
            Without<PlayerSprite>,
            With<behavior::component::FloorItemCollidable>,
        ),
    >,
    mut gs: gear::GearStuff,
) {
    for (mut player_gear, player_pos, player) in players.iter_mut() {
        if keyboard_input.just_pressed(player.controls.drop) {
            // Take the held object from the player's gear (this removes it temporarily)
            if let Some(held_object) = player_gear.held_item.take() {
                // Check for valid Drop location
                let target_tile = player_pos.to_board_position();
                let is_valid_tile = gs
                    .bf
                    .collision_field
                    .get(&target_tile)
                    .map(|col| col.player_free)
                    .unwrap_or(false);
                // Check for object obstruction
                let is_obstructed = objects.iter().any(|(entity, object_pos)| {
                    // Skip checking the held object itself
                    if entity == held_object.entity {
                        return false;
                    }
                    // **Collision Check:**
                    target_tile.to_position().distance(object_pos) < 0.5
                });

                // Only drop if valid
                if is_valid_tile && !is_obstructed {
                    // Retrieve the ORIGINAL entity of the held object
                    if let Ok((_, mut position)) = objects.get_mut(held_object.entity) {
                        // Update the object's Position component
                        *position = target_tile.to_position();

                        // Play "Drop" sound effect
                        gs.play_audio("sounds/item-drop-clunk.ogg".into(), 1.0, player_pos);
                    } else {
                        warn!("Failed to retrieve components from held object entity.");

                        // Put the object back in the player's gear if we can't drop it
                        player_gear.held_item = Some(held_object);
                    }
                } else {
                    // --- Invalid Drop Handling ---
                    // Play "Invalid Drop" sound effect
                    gs.play_audio("sounds/invalid-action-buzz.ogg".into(), 0.3, player_pos);

                    // Put the object back in the player's gear
                    player_gear.held_item = Some(held_object);
                }
            }
        }
    }
}

/// Allows the player to hide in a designated hiding spot.
///
/// This system checks if the player is pressing the 'activate' key and is near a valid hiding spot.
/// If so, the player character enters the hiding spot, becoming partially hidden. A visual overlay is added to the hiding spot to indicate the player's presence.
/// Note that the player's transparency while hiding is not yet fully implemented.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn hide_player(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<
        (Entity, &mut PlayerSprite, &mut Position, &PlayerGear),
        (Without<Hiding>, Without<Behavior>),
    >,
    hiding_spots: Query<(Entity, &Position, &Behavior), Without<PlayerSprite>>,
    asset_server: Res<AssetServer>,
    mut gs: gear::GearStuff,
    time: Res<Time>,
    mut hold_timers: Local<HashMap<Entity, Timer>>,
) {
    for (player_entity, player, mut player_pos, player_gear) in players.iter_mut() {
        // Get the player's hold timer or create a new one
        let timer = hold_timers
            .entry(player_entity)
            .or_insert_with(|| Timer::from_seconds(0.3, TimerMode::Once));

        if keyboard_input.pressed(player.controls.activate) {
            if player_gear.held_item.is_some() {
                // Player cannot hide while carrying furniture.
                continue;
            }
            // Using 'activate' for hiding
            // Find a hiding spot near the player
            if let Some((hiding_spot_entity, hiding_spot_pos, _)) = hiding_spots
                .iter()
                .filter(|(_, _, behavior)| behavior.p.object.hidingspot) // Manually filter for hiding spots
                .find(|(_, hiding_spot_pos, _)| player_pos.distance(hiding_spot_pos) < 1.0)
            {
                // Key is held down, tick the timer
                timer.tick(time.delta());

                if !timer.finished() {
                    continue;
                }
                timer.reset();

                // Add the Hiding component to the player
                commands
                    .entity(player_entity)
                    .insert(Hiding {
                        hiding_spot: hiding_spot_entity,
                    })
                    .insert(MapColor {
                        color: css::DARK_GRAY.with_alpha(0.5).into(),
                    });

                player_pos.x = (player_pos.x + hiding_spot_pos.x) / 2.0;
                player_pos.y = (player_pos.y + hiding_spot_pos.y) / 2.0;

                // Play "Hide" sound effect
                gs.play_audio("sounds/hide-rustle.ogg".into(), 1.0, &player_pos);

                // Add Visual Overlay
                commands.entity(hiding_spot_entity).with_children(|parent| {
                    parent.spawn(SpriteBundle {
                        texture: asset_server.load("img/hiding_overlay.png"),
                        transform: Transform::from_xyz(0.0, 0.0, 0.02)
                            .with_scale(Vec3::new(0.20, 0.20, 0.20)), // Position relative to parent
                        sprite: Sprite {
                            color: css::WHITE.with_alpha(0.4).into(),
                            ..default()
                        },
                        ..default()
                    });
                });
            }
        } else {
            timer.reset();
        }
    }
}

/// Allows the player to leave a hiding spot.
///
/// This system checks if the player is pressing the 'activate' key and is currently hiding.
/// If so, the player character exits the hiding spot, their visibility is restored, and the visual overlay is removed from the hiding spot.
pub fn unhide_player(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(
        Entity,
        &mut PlayerSprite,
        &mut Transform,
        &mut Visibility,
        &Hiding,
    )>,
) {
    for (player_entity, player, _, _visibility, hiding) in players.iter_mut() {
        if keyboard_input.just_pressed(player.controls.activate) {
            // Using 'activate' for unhiding
            // Remove the Hiding component
            commands.entity(player_entity).remove::<Hiding>();

            // Reset player sprite animation
            // TODO: Define default animation
            // For now, let's just set it back to the standing animation (index 32)
            commands
                .entity(player_entity)
                .insert(AnimationTimer::from_range(
                    Timer::from_seconds(0.20, TimerMode::Repeating),
                    vec![32],
                ))
                .insert(MapColor {
                    color: Color::WHITE.with_alpha(1.0),
                });

            // Reset player position
            // TODO: Consider using the hiding spot's position
            // For now, let's just leave the position as is.

            // Reset player visibility
            // *visibility = Visibility::Visible;

            // --- Remove Visual Overlay ---
            commands.entity(hiding.hiding_spot).despawn_descendants();
        }
    }
}

fn lose_sanity(
    time: Res<Time>,
    mut timer: Local<utils::PrintingTimer>,
    mut mean_sound: Local<MeanSound>,
    mut qp: Query<(&mut PlayerSprite, &Position)>,
    bf: Res<BoardData>,
    roomdb: Res<board::RoomDB>,
    difficulty: Res<CurrentDifficulty>, // Access the difficulty settings
) {
    timer.tick(time.delta());

    let dt = time.delta_seconds();
    for (mut ps, pos) in &mut qp {
        let bpos = pos.to_board_position();
        let lux = bf
            .light_field
            .get(&bpos)
            .map(|x| x.lux)
            .unwrap_or(2.0)
            .sqrt()
            + 0.001;
        let temp = bf.temperature_field.get(&bpos).cloned().unwrap_or(2.0);
        let f_temp = (temp - bf.ambient_temp / 2.0).clamp(0.0, 10.0) + 1.0;
        let f_temp2 = (bf.ambient_temp / 2.0 - temp).clamp(0.0, 10.0) + 1.0;
        let mut sound = 0.0;
        for bpos in bpos.xy_neighbors(3).iter() {
            sound += bf
                .sound_field
                .get(bpos)
                .map(|x| x.iter().map(|y| y.length()).sum::<f32>())
                .unwrap_or_default()
                * 10.0;
        }
        const MASS: f32 = 10.0;

        if roomdb.room_tiles.contains_key(&bpos) {
            mean_sound.0 =
                ((sound * dt + mean_sound.0 * MASS) / (MASS + dt)).clamp(0.00000001, 100000.0);
        } else {
            // prevent sanity from being lost outside of the location.
            mean_sound.0 /= 1.8_f32.powf(dt);
        }
        let crazy =
            lux.recip() / f_temp * f_temp2 * mean_sound.0 * 10.0 + mean_sound.0 / f_temp * f_temp2;
        let sanity_recover: f32 = if ps.sanity() < difficulty.0.starting_sanity {
            4.0 / 100.0 / difficulty.0.sanity_drain_rate
        } else {
            0.0
        };
        ps.crazyness +=
            (crazy.clamp(0.000000001, 10000000.0).sqrt() * 0.2 * difficulty.0.sanity_drain_rate
                - sanity_recover * ps.crazyness / (1.0 + mean_sound.0 * 10.0))
                * dt;
        if ps.crazyness < 0.0 {
            ps.crazyness = 0.0;
        }
        ps.mean_sound = mean_sound.0;
        if ps.health < 100.0 && ps.health > 0.0 {
            ps.health += (0.1 * dt + (1.0 - ps.health / 100.0) * dt * 10.0)
                * difficulty.0.health_recovery_rate;
        }
        if ps.health > 100.0 {
            ps.health = 100.0;
        }
        if timer.just_finished() && DEBUG_PLAYER {
            dbg!(ps.sanity(), mean_sound.0, ps.health);
        }
    }
}

fn recover_sanity(
    time: Res<Time>,
    mut qp: Query<&mut PlayerSprite>,
    gc: Res<GameConfig>,
    mut timer: Local<utils::PrintingTimer>,
    difficulty: Res<CurrentDifficulty>, // Access the difficulty settings
) {
    // Current player recovers sanity while in the truck.
    let dt = time.delta_seconds();
    timer.tick(time.delta());

    for mut ps in &mut qp {
        if ps.id == gc.player_id {
            // --- Gradual Health Recovery ---
            const HEALTH_RECOVERY_RATE: f32 = 2.0; // Health points recovered per second

            if ps.health < 100.0 {
                ps.health += HEALTH_RECOVERY_RATE * dt;
                ps.health = ps.health.min(100.0); // Clamp health to a maximum of 100%
            }
            if ps.sanity() < difficulty.0.starting_sanity {
                ps.crazyness /= 1.07_f32.powf(dt);
            }
            if timer.just_finished() {
                dbg!(ps.sanity());
            }
        }
    }
}

pub fn visual_health(
    qp: Query<&PlayerSprite>,
    gc: Res<GameConfig>,
    mut qb: Query<(
        Option<&mut UiImage>,
        &mut BackgroundColor,
        &DamageBackground,
    )>,
) {
    for player in &qp {
        if player.id != gc.player_id {
            continue;
        }
        let health = (player.health.clamp(0.0, 100.0) / 100.0).clamp(0.0, 1.0);
        let crazyness = (1.0 - player.sanity() / 100.0).clamp(0.0, 1.0);
        for (mut o_uiimage, mut bgcolor, dmg) in &mut qb {
            let rhealth = (1.0 - health).powf(dmg.exp);
            let crazyness = crazyness.powf(dmg.exp);
            let alpha = ((rhealth * 10.0).clamp(0.0, 0.3) + rhealth.powi(2) * 0.7 + crazyness)
                .clamp(0.0, 1.0);
            let rhealth2 = (1.0 - alpha * 0.9).clamp(0.0001, 1.0);
            let red = f32::tanh(rhealth * 2.0).clamp(0.0, 1.0) * rhealth2;
            let dst_color = Color::srgba(red, 0.0, 0.0, alpha);

            let old_color = o_uiimage.as_ref().map(|x| x.color).unwrap_or(bgcolor.0);
            let new_color = maplight::lerp_color(old_color, dst_color, 0.2);
            if old_color != new_color {
                dbg!(&new_color);
                if let Some(uiimage) = o_uiimage.as_mut() {
                    uiimage.color = new_color;
                } else {
                    bgcolor.0 = new_color;
                }
            }
        }
    }
}

/// Updates the position of the player's held object to match the player's position.
///
/// This system ensures that the held object visually follows the player when they move.
/// It also slightly elevates the object's Z position to create a visual indication
/// that the object is being held. Additionally, it plays a scraping sound effect
/// when the player moves while holding a movable object, with a cooldown to prevent
/// the sound from playing too frequently.
#[allow(clippy::type_complexity)]
pub fn update_held_object_position(
    mut objects: Query<(&mut Position, &Behavior), Without<PlayerSprite>>,
    players: Query<(&Position, &PlayerGear, &board::Direction), With<PlayerSprite>>,
    mut gs: gear::GearStuff,
    mut last_sound_time: Local<f32>,
) {
    let current_time = gs.time.elapsed_seconds();

    for (player_pos, player_gear, direction) in players.iter() {
        if let Some(held_object) = &player_gear.held_item {
            if let Ok((mut object_pos, behavior)) = objects.get_mut(held_object.entity) {
                // Match the object's position to the player's position
                *object_pos = *player_pos;

                // Slightly elevate the object's Z position
                const OBJECT_ELEVATION: f32 = 0.1;
                object_pos.z += OBJECT_ELEVATION;

                // --- Play Scraping Sound if Object is Movable and Player is Moving ---
                if behavior.p.object.movable
                   && direction.distance() > 75.0 // Player is moving
                   && current_time - *last_sound_time > 2.0
                // Sound cooldown
                {
                    // Play "Move" sound effect
                    gs.play_audio("sounds/item-move-scrape.ogg".into(), 0.1, player_pos);

                    // Update last sound time
                    *last_sound_time = current_time;
                }
            }
        }
    }
}

/// System for deploying a piece of gear from the player's right hand into the game world.
pub fn deploy_gear(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&mut PlayerGear, &Position, &PlayerSprite, &board::Direction)>,
    mut commands: Commands,
    q_collidable: Query<(Entity, &Position), With<behavior::component::FloorItemCollidable>>,
    mut gs: gear::GearStuff,
    handles: Res<root::GameAssets>,
) {
    for (mut player_gear, player_pos, player, dir) in players.iter_mut() {
        if keyboard_input.just_pressed(player.controls.drop)
            && player_gear.right_hand.kind.is_some()
            && player_gear.held_item.is_none()
        {
            let deployed_gear = DeployedGear { direction: *dir };
            let target_tile = player_pos.to_board_position();
            let is_valid_tile = gs
                .bf
                .collision_field
                .get(&target_tile)
                .map(|col| col.player_free)
                .unwrap_or(false);

            let is_obstructed = q_collidable
                .iter()
                .any(|(_entity, object_pos)| target_tile.to_position().distance(object_pos) < 0.5);
            if is_valid_tile && !is_obstructed {
                let scoord = player_pos.to_screen_coord();
                let gear_sprite = SpriteSheetBundle {
                    texture: handles.images.gear.clone(),
                    atlas: TextureAtlas {
                        layout: handles.images.gear_atlas.clone(),
                        index: player_gear.right_hand.get_sprite_idx() as usize,
                    },
                    transform: Transform::from_xyz(scoord.x, scoord.y, scoord.z + 0.01)
                        .with_scale(Vec3::new(0.25, 0.25, 0.25)), // Initial scaling factor
                    ..Default::default()
                };

                commands
                    .spawn(gear_sprite)
                    .insert(deployed_gear)
                    .insert(*player_pos)
                    .insert(behavior::component::FloorItemCollidable)
                    .insert(game::GameSprite)
                    .insert(DeployedGearData {
                        gear: player_gear.right_hand.take(),
                    });
                player_gear.cycle();
                // Play "Drop Item" sound effect (reused for gear deployment)
                gs.play_audio("sounds/item-drop-clunk.ogg".into(), 1.0, player_pos);
            } else {
                // Play "Invalid Drop" sound effect
                gs.play_audio("sounds/invalid-action-buzz.ogg".into(), 0.3, player_pos);
            }
        }
    }
}

/// System for retrieving deployed gear and adding it to the player's right hand.
pub fn retrieve_gear(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&Position, &PlayerSprite, &mut PlayerGear)>,
    q_deployed: Query<(Entity, &Position, &DeployedGearData)>,
    mut commands: Commands,
    mut gs: gear::GearStuff,
) {
    // FIXME: This code, along with grabbing items are in conflict. It will be
    // possible for a player to grab equipment from the floor and a location
    // item at the same time if they are close enough for a well placed player.
    // This needs to be solved, likely by handling the keypress event in
    // one single system, then routing the remaining stuff to do via an Event
    // to the system that handles that exact thing.
    for (player_pos, player, mut player_gear) in players.iter_mut() {
        if keyboard_input.just_pressed(player.controls.grab) {
            // Find the closest deployed gear
            let mut closest_gear: Option<(Entity, f32)> = None;
            for (entity, gear_pos, _) in q_deployed.iter() {
                let distance = player_pos.distance(gear_pos);
                if distance < 1.2 {
                    if let Some((_, closest_distance)) = closest_gear {
                        if distance < closest_distance {
                            closest_gear = Some((entity, distance));
                        }
                    } else {
                        closest_gear = Some((entity, distance));
                    }
                }
            }

            // Retrieve the closest gear
            if let Some((closest_gear_entity, _)) = closest_gear {
                if let Ok((_, _, deployed_gear_data)) = q_deployed.get(closest_gear_entity) {
                    // Inventory Shifting Logic:
                    if player_gear.right_hand.kind.is_some() {
                        // Right hand is occupied, try to shift to inventory
                        if let Some(empty_slot_index) = player_gear
                            .inventory
                            .iter()
                            .position(|gear| gear.kind.is_none())
                        {
                            // Move right-hand gear to the empty slot
                            player_gear.inventory[empty_slot_index] = gear::Gear {
                                kind: player_gear.right_hand.kind.clone(),
                            };
                        } else {
                            // No empty slot - play invalid action sound and skip retrieval
                            gs.play_audio("sounds/invalid-action-buzz.ogg".into(), 0.3, player_pos);
                            return;
                        }
                    }

                    // Now the right hand is free, proceed with retrieval
                    player_gear.right_hand = deployed_gear_data.gear.clone();
                    commands.entity(closest_gear_entity).despawn();
                    // Play "Grab Item" sound effect (reused for gear retrieval)
                    gs.play_audio("sounds/item-pickup-whoosh.ogg".into(), 1.0, player_pos);
                }
            }
            // --
        }
    }
}

#[derive(Default)]
struct MeanSound(f32);

pub fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            keyboard_player,
            lose_sanity,
            visual_health,
            animate_sprite,
            update_held_object_position,
            deploy_gear,
            retrieve_gear,
        )
            .run_if(in_state(root::GameState::None)),
    )
    .add_systems(
        Update,
        recover_sanity.run_if(in_state(root::GameState::Truck)),
    )
    .add_systems(Update, grab_object.run_if(in_state(root::GameState::None)))
    .add_systems(Update, drop_object.run_if(in_state(root::GameState::None)))
    .add_systems(Update, hide_player.run_if(in_state(root::GameState::None)))
    .add_systems(
        Update,
        unhide_player.run_if(in_state(root::GameState::None)),
    );
}
