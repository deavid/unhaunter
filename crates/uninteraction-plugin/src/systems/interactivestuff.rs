use unbehavior::behavior::Behavior;
use unbehavior::behavior::Interactive;
use unbehavior::components::RoomState;
use unbehavior::roomdb::{RoomStateMap, RoomTopology};
use unevents_core::events::roomchanged::InteractionExecutionType;
use unevents_core::events::sound::SoundEvent;
use unrender_std::board::spritedb::SpriteDB;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::position::Position;

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
    /// Database of sprites for map tiles. Used to retrieve the correct behavior variant
    /// for interactive objects when their state changes.
    pub bf: Option<Res<'w, SpriteDB>>,
    /// Used to insert updated Behavior components on entities.
    pub commands: Commands<'w, 's>,
    /// Event writer for sending sound events.
    pub sound_events: MessageWriter<'w, SoundEvent>,
    /// Database of room data, used to track the state of rooms and update interactive
    /// objects accordingly.
    pub roomtopo: ResMut<'w, RoomTopology>,
    pub roomstate: ResMut<'w, RoomStateMap>,
}

impl InteractiveStuff<'_, '_> {
    /// Internal helper to update the Behavior component of an entity to match a specific
    /// tile identifier. Visual updates are handled reactively by the Renderer system.
    fn apply_behavior_update(
        &mut self,
        entity: Entity,
        tuid: &(String, u32),
        current_behavior: &Behavior,
    ) {
        let Some(bf) = self.bf.as_ref() else {
            warn!(
                "apply_behavior_update: SpriteDB is None for entity {:?} tuid {:?} - Behavior will NOT be updated",
                entity, tuid
            );
            return;
        };
        let other = bf
            .map_tile
            .get(tuid)
            .expect("Tile UID not found in SpriteDB");
        let mut beh = other.behavior.clone();
        beh.flip(current_behavior.p.flip);

        let mut e_commands = self.commands.get_entity(entity).unwrap();
        info!(
            "apply_behavior_update: inserting new Behavior for entity {:?} tuid {:?}",
            entity, tuid
        );
        e_commands.insert(beh);
    }

    /// Synchronizes the entity's state with the current RoomStateMap state.
    ///
    /// Checks the `RoomState` of the room the entity belongs to. If the entity's
    /// current behavior state (`beh.state()`) does not match the room's stored
    /// state, it updates the entity to match.
    ///
    /// Returns `true` if the entity was updated.
    pub fn synchronize_entity(
        &mut self,
        entity: Entity,
        item_pos: &Position,
        behavior: &Behavior,
        room_state: &RoomState,
    ) -> bool {
        let item_bpos = item_pos.to_board_position();
        let item_roombpos = BoardPosition {
            x: item_bpos.x + room_state.room_delta.x,
            y: item_bpos.y + room_state.room_delta.y,
            z: item_bpos.z + room_state.room_delta.z,
        };
        let room_name = self
            .roomtopo
            .room_tiles
            .get(&item_roombpos)
            .cloned()
            .unwrap_or_default();

        let Some(main_room_state) = self.roomstate.room_state.get(&room_name) else {
            return false;
        };

        if behavior.state() == *main_room_state {
            return false;
        }

        // We need to find the correct variant for this state.
        let cvo = behavior.key_cvo();
        let Some(bf) = self.bf.as_ref() else {
            return false;
        };
        let variants = bf.cvo_idx.get(&cvo).cloned().unwrap_or_default();

        for variant_tuid in variants.iter() {
            let is_match = bf
                .map_tile
                .get(variant_tuid)
                .map(|other| other.behavior.state() == *main_room_state)
                .unwrap_or(false);

            if is_match {
                trace!(
                    "synchronize_entity: Syncing entity {:?} to state {:?} (tuid={:?})",
                    entity, main_room_state, variant_tuid
                );
                self.apply_behavior_update(entity, variant_tuid, behavior);
                return true;
            }
        }
        false
    }

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
        force_tuid: Option<u32>,
    ) -> bool {
        if ietype == InteractionExecutionType::ReadRoomState {
            warn!(
                "execute_interaction: ReadRoomState is deprecated, use RoomStateSyncEvent instead."
            );
        }
        trace!(
            "execute_interaction: entity={:?}, ietype={:?}, force_tuid={:?}",
            entity, ietype, force_tuid
        );
        let item_bpos = item_pos.to_board_position();
        let tuid = behavior.key_tuid();
        let cvo = behavior.key_cvo();

        let Some(bf) = self.bf.as_ref() else {
            return false;
        };
        let variants = bf.cvo_idx.get(&cvo).cloned().unwrap_or_default();
        for other_tuid in variants.iter() {
            if let Some(ftuid) = force_tuid {
                if other_tuid.1 != ftuid {
                    continue;
                }
            } else if *other_tuid == tuid {
                continue;
            }
            let (beh_state, _other_tileset, _other_tileuid, other_behavior) = {
                let other = bf.map_tile.get(other_tuid).unwrap();
                (
                    other.behavior.state(),
                    other.behavior.cfg().tileset.clone(),
                    other.behavior.cfg().tileuid,
                    other.behavior.clone(),
                )
            };

            // In case it is connected to a room, we need to change room state.
            if let Some(room_state) = room_state {
                let item_roombpos = BoardPosition {
                    x: item_bpos.x + room_state.room_delta.x,
                    y: item_bpos.y + room_state.room_delta.y,
                    z: item_bpos.z + room_state.room_delta.z,
                };
                let room_name = self
                    .roomtopo
                    .room_tiles
                    .get(&item_roombpos)
                    .cloned()
                    .unwrap_or_default();

                match ietype {
                    InteractionExecutionType::ChangeState => {
                        if let Some(main_room_state) = self.roomstate.room_state.get_mut(&room_name)
                        {
                            *main_room_state = beh_state.clone();
                        }
                    }
                    InteractionExecutionType::ReadRoomState => {
                        if let Some(main_room_state) = self.roomstate.room_state.get(&room_name)
                            && *main_room_state != beh_state
                        {
                            continue;
                        }
                    }
                }
            }

            trace!(
                "execute_interaction: Changing entity {:?} state to tuid {:?}",
                entity, other_tuid
            );

            self.apply_behavior_update(entity, other_tuid, behavior);

            if ietype == InteractionExecutionType::ChangeState
                && let Some(interactive) = interactive
            {
                let sound_file = interactive.sound_for_moving_into_state(&other_behavior);
                self.sound_events.write(SoundEvent {
                    sound_file,
                    volume: 1.0,
                    position: Some(*item_pos),
                    broadcast: true,
                });
            }
            return true;
        }
        if let Some(ftuid) = force_tuid {
            // This currently happens because the host seems to send an interaction per tile position.
            warn!(
                "execute_interaction: attempted to set sprite tuid {:?} to {:?} for entity {:?} but that variant does not exist for that sprite",
                ftuid, cvo, entity
            );
        }
        false
    }
}
