use crate::uncore_board::{RoomDB, SpriteDB};
use crate::uncore_root::GameState;
use uncore::behavior::component::{Interactive, RoomState};
use uncore::behavior::Behavior;
use uncore::components::board::boardposition::BoardPosition;
use uncore::components::board::position::Position;
use uncore::events::roomchanged::InteractionExecutionType;
use unstd::materials::CustomMaterial1;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

/// The `InteractiveStuff` system handles interactions between the player and
/// interactive objects in the game world, such as doors, switches, lamps, and the
/// van entry.
///
/// This system centralizes the logic for:
///
/// * Changing the state of interactive objects based on player interaction or room
///   state.
///
/// * Playing appropriate sound effects for different interactions.
///
/// * Triggering transitions to the truck UI when the player enters the van.
#[derive(SystemParam)]
pub struct InteractiveStuff<'w, 's> {
    /// Database of sprites for map tiles. Used to retrieve alternative sprites for
    /// interactive objects.
    pub bf: Res<'w, SpriteDB>,
    /// Used to spawn sound effects and potentially other entities related to
    /// interactions.
    pub commands: Commands<'w, 's>,
    /// Access to the asset server for loading sound effects.
    pub asset_server: Res<'w, AssetServer>,
    /// Access to the materials used for rendering map tiles. Used to update tile
    /// visuals when object states change.
    pub materials1: ResMut<'w, Assets<CustomMaterial1>>,
    /// Database of room data, used to track the state of rooms and update interactive
    /// objects accordingly.
    pub roomdb: ResMut<'w, RoomDB>,
    /// Controls the transition to different game states, such as the truck UI.
    pub game_next_state: ResMut<'w, NextState<GameState>>,
}

impl InteractiveStuff<'_, '_> {
    /// Executes an interaction with an interactive object.
    ///
    /// This method determines the object's new state based on the type of interaction,
    /// updates its `Behavior` component, plays the corresponding sound effect, and
    /// updates the room state if applicable.
    ///
    /// # Parameters:
    ///
    /// * `entity`: The entity of the interactive object.
    ///
    /// * `item_pos`: The position of the interactive object in the game world.
    ///
    /// * `interactive`: The `Interactive` component of the object, if present.
    ///
    /// * `behavior`: The `Behavior` component of the object.
    ///
    /// * `room_state`: The `RoomState` component of the object, if present.
    ///
    /// * `ietype`: The type of interaction being executed (`ChangeState` or
    ///   `ReadRoomState`).
    ///
    /// # Returns:
    ///
    /// `true` if the interaction resulted in a change to the object's state, `false`
    /// otherwise.
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
                self.commands
                    .spawn(AudioPlayer::<AudioSource>(
                        self.asset_server.load(sound_file),
                    ))
                    .insert(PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Despawn,
                        volume: bevy::audio::Volume::new(1.0),
                        speed: 1.0,
                        paused: false,
                        spatial: false,
                        spatial_scale: None,
                    });
            }
            self.game_next_state.set(GameState::Truck);
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

                // dbg!(&room_state, &item_roombpos); dbg!(&room_name);
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
            let b = other.bundle.clone();
            let mat = self.materials1.get(&b.material).unwrap().clone();
            let mat = self.materials1.add(mat);
            e_commands.insert(MeshMaterial2d(mat));

            e_commands.insert(beh);
            if ietype == InteractionExecutionType::ChangeState {
                if let Some(interactive) = interactive {
                    let sound_file = interactive.sound_for_moving_into_state(&other.behavior);
                    self.commands
                        .spawn(AudioPlayer::<AudioSource>(
                            self.asset_server.load(sound_file),
                        ))
                        .insert(PlaybackSettings {
                            mode: bevy::audio::PlaybackMode::Despawn,
                            volume: bevy::audio::Volume::new(1.0),
                            speed: 1.0,
                            paused: false,
                            spatial: false,
                            spatial_scale: None,
                        });
                }
            }
            return true;
        }
        false
    }
}
