//! Phase 4: Gear replication and distributed authority (Pillar 5).
//!
//! This module handles:
//! - Registering gear components for direct replication.
//! - Orphan & Re-Adopt logic: sync entity parenting between client and server.

use bevy::prelude::*;
use bevy_replicon::prelude::AppRuleExt;
use ungear_core::components::core::*;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::{PlayerGear, HeldObject};
use ungearitems_core::components::emfmeter::EMFMeter;
use ungearitems_core::components::flashlight::Flashlight;
use ungearitems_core::components::repellentflask::RepellentFlask;
use ungearitems_core::components::sage::SageBundleData;
use ungearitems_core::components::spiritbox::SpiritBox;
use ungearitems_core::components::thermometer::Thermometer;
use unreplicon_core::ownership::{Owner, LocallyOwned};
use unplayer_core::components::MainPlayer;
use crate::systems::conditions::is_pure_client;

pub(super) fn app_setup(app: &mut App) {
    // Register gear components for replication.
    app.replicate::<ItemName>();
    app.replicate::<ItemDescription>();
    app.replicate::<GearSprite>();
    app.replicate::<Electronic>();
    app.replicate::<Battery>();
    app.replicate::<EvidenceSensor>();
    app.replicate::<PerceivedClarity>();
    app.replicate::<Handheld>();
    app.replicate::<StatusText>();
    app.replicate::<DeployedGear>();
    app.replicate::<HeldObject>();

    // Register item-specific components.
    app.replicate::<Flashlight>();
    app.replicate::<Thermometer>();
    app.replicate::<EMFMeter>();
    app.replicate::<SpiritBox>();
    app.replicate::<SageBundleData>();
    app.replicate::<RepellentFlask>();

    // Server-side: Manage gear parenting based on PlayerGear content.
    app.add_systems(
        Update,
        sync_gear_parenting_server.run_if(resource_exists::<untypes_core::roles::AuthorityRole>),
    );

    // Client-side: Local player's held gear is LocallyOwned.
    app.add_systems(
        Update,
        sync_local_gear_authority.run_if(resource_exists::<untypes_core::roles::LocalPlayerRole>),
    );

    // Client-side: Sync parenting for remote players.
    app.add_systems(
        Update,
        sync_gear_parenting_client.run_if(is_pure_client),
    );
}

/// Server: Ensure that any gear entity listed in a player's `PlayerGear` is correctly
/// parented to that player entity, and has the correct `Owner`.
fn sync_gear_parenting_server(
    q_players: Query<(Entity, &Owner, &PlayerGear), Changed<PlayerGear>>,
    mut commands: Commands,
    q_gear: Query<(Option<&ChildOf>, Option<&Owner>), With<Handheld>>,
) {
    for (player_entity, player_owner, gear) in q_players.iter() {
        let held_entities = gear.left_hand.iter()
            .chain(gear.right_hand.iter())
            .chain(gear.inventory.iter())
            .copied();

        for gear_entity in held_entities {
            let Ok((maybe_child_of, maybe_owner)) = q_gear.get(gear_entity) else { continue; };

            // Ensure correct parenting.
            if maybe_child_of.map(|c| c.parent()) != Some(player_entity) {
                commands.entity(player_entity).add_child(gear_entity);
            }

            // Ensure correct owner (matches the player who holds it).
            if let Some(owner) = maybe_owner {
                if owner.0 != player_owner.0 {
                    commands.entity(gear_entity).insert(Owner(player_owner.0));
                }
            } else {
                commands.entity(gear_entity).insert(Owner(player_owner.0));
            }
        }
    }
}

/// Client: Mark gear entities held by the local player as `LocallyOwned`.
fn sync_local_gear_authority(
    q_local_player: Query<&PlayerGear, (With<MainPlayer>, Changed<PlayerGear>)>,
    q_gear: Query<Entity, (With<Handheld>, Without<LocallyOwned>)>,
    mut commands: Commands,
) {
    let Ok(gear) = q_local_player.single() else { return; };
    let held_entities = gear.left_hand.iter()
        .chain(gear.right_hand.iter())
        .chain(gear.inventory.iter())
        .copied();

    for gear_entity in held_entities {
        if q_gear.get(gear_entity).is_ok() {
            commands.entity(gear_entity).insert(LocallyOwned);
        }
    }
}

/// Client: Mirror parenting for remote players based on their replicated `PlayerGear`.
fn sync_gear_parenting_client(
    q_remote_players: Query<(Entity, &PlayerGear), (Without<MainPlayer>, Changed<PlayerGear>)>,
    mut commands: Commands,
    q_gear: Query<Option<&ChildOf>, With<Handheld>>,
) {
    for (player_entity, gear) in q_remote_players.iter() {
        let held_entities = gear.left_hand.iter()
            .chain(gear.right_hand.iter())
            .chain(gear.inventory.iter())
            .copied();

        for gear_entity in held_entities {
            if let Ok(maybe_child_of) = q_gear.get(gear_entity) {
                if maybe_child_of.map(|c| c.parent()) != Some(player_entity) {
                    commands.entity(player_entity).add_child(gear_entity);
                }
            }
        }
    }
}
