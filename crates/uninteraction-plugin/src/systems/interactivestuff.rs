use unbehavior::behavior::Behavior;
use unbehavior::behavior::Interactive;
use unbehavior::components::RoomState;
use unbehavior::roomdb::RoomDB;
use unevents_core::events::roomchanged::InteractionExecutionType;
use unevents_core::events::sound::SoundEvent;
use uninteraction_core::interaction::Authority;
use unnet_core::messages::{NetworkDataEvent, NetworkMessage};
use unrender_std::board::spritedb::SpriteDB;
use unrender_std::materials::CustomMaterial1;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::position::Position;
use untypes_core::states::GameState;

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
    /// Event writer for sending sound events.
    pub sound_events: MessageWriter<'w, SoundEvent>,
    /// Access to the asset server for loading sound effects.
    pub asset_server: Res<'w, AssetServer>,
    /// Access to the materials used for rendering map tiles. Used to update tile
    /// visuals when object states change.
    pub materials1: ResMut<'w, Assets<CustomMaterial1>>,
    /// ID of the local player.
    pub local_player: Res<'w, unnet_core::resources::LocalPlayer>,
    /// Database of room data, used to track the state of rooms and update interactive
    /// objects accordingly.
    pub roomdb: ResMut<'w, RoomDB>,
    /// Controls the transition to different game states, such as the truck UI.
    pub game_next_state: ResMut<'w, NextState<GameState>>,
    /// Event writer for sending network messages.
    pub net_events: MessageWriter<'w, NetworkDataEvent>,
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
        authority: Authority,
        force_tuid: Option<u32>,
    ) -> bool {
        debug!(
            "execute_interaction: entity={:?}, ietype={:?}, authority={:?}, force_tuid={:?}",
            entity, ietype, authority, force_tuid
        );
        let item_bpos = item_pos.to_board_position();
        let tuid = behavior.key_tuid();
        let cvo = behavior.key_cvo();
        if behavior.is_van_entry() {
            if ietype != InteractionExecutionType::ChangeState {
                return false;
            }
            match authority {
                Authority::Host => {
                    if let Some(interactive) = interactive {
                        let sound_file = interactive.sound_for_moving_into_state(behavior);
                        self.sound_events.write(SoundEvent {
                            sound_file,
                            volume: 1.0,
                            position: Some(*item_pos),
                        });
                    }
                    self.game_next_state.set(GameState::Truck);
                }
                Authority::Client => {
                    self.net_events.write(NetworkDataEvent {
                        message: NetworkMessage::RequestTruckEntry,
                    });
                }
            }
            return false;
        }

        if force_tuid.is_none() && authority == Authority::Client {
            if let Some(player_id) = self.local_player.0 {
                debug!("Client: Requesting interaction at {:?}", item_bpos);
                self.net_events.write(NetworkDataEvent {
                    message: NetworkMessage::InteractionRequest {
                        player_id,
                        position: [item_bpos.x as i32, item_bpos.y as i32, item_bpos.z as i32],
                        interaction_type: ietype,
                    },
                });
            }
            return false;
        }

        for other_tuid in self.bf.cvo_idx.get(&cvo).unwrap().iter() {
            if let Some(ftuid) = force_tuid {
                if other_tuid.1 != ftuid {
                    continue;
                }
            } else if *other_tuid == tuid {
                continue;
            }
            let mut e_commands = self.commands.get_entity(entity).unwrap();
            let other = self.bf.map_tile.get(other_tuid).unwrap();
            let mut beh = other.behavior.clone();
            beh.flip(behavior.p.flip);

            debug!(
                "execute_interaction: Changing entity {:?} state to tuid {:?} (authority={:?})",
                entity, other_tuid, authority
            );

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

                match ietype {
                    InteractionExecutionType::ChangeState => {
                        if authority == Authority::Host
                            && let Some(main_room_state) =
                                self.roomdb.room_state.get_mut(&room_name)
                        {
                            *main_room_state = beh.state();
                        }
                    }
                    InteractionExecutionType::ReadRoomState => {
                        if let Some(main_room_state) = self.roomdb.room_state.get(&room_name)
                            && *main_room_state != beh.state()
                        {
                            continue;
                        }
                    }
                }
            }
            let b = other.bundle.clone();
            let mat = self.materials1.get(&b.material).unwrap().clone();
            let mat = self.materials1.add(mat);
            e_commands.insert(MeshMaterial2d(mat));

            e_commands.insert(beh);
            if ietype == InteractionExecutionType::ChangeState
                && let Some(interactive) = interactive
                && authority == Authority::Host
            {
                let sound_file = interactive.sound_for_moving_into_state(&other.behavior);
                self.sound_events.write(SoundEvent {
                    sound_file,
                    volume: 1.0,
                    position: Some(*item_pos),
                });
            }
            return true;
        }
        false
    }
}
