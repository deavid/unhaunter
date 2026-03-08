use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use unbehavior::behavior::{Behavior, Interactive};
use unbehavior::components::{RoomState, TmxEntityId};
use unrender_std::components::game::GameSprite;
use unrender_std::materials::CustomMaterial1;
use unspatial_core::boardposition::MapEntityFieldBPos;
use unspatial_core::position::Position;
use untypes_core::roles::is_pure_client;

pub(super) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        stitch_map_entities.run_if(is_pure_client),
    );
}

/// Client-side stitcher: when a server-replicated dynamic entity arrives (carrying
/// `TmxEntityId` + `Replicated`), find the matching locally-spawned placeholder,
/// transfer ALL components (visual and logic) to the replicated entity, then despawn
/// the placeholder.
///
/// Run condition: `is_pure_client` only. On a host node the server and client share
/// the same entities, so no stitching is required or safe.
///
/// ## Why logic components must be transferred
///
/// Because `Behavior` is not yet replicated, the server entity arrives on the client
/// without `Behavior`, `Interactive`, `RoomState`, or `MapEntityFieldBPos`. The client's
/// `apply_remote_interaction` system looks up interactive entities via `MapEntityFieldBPos`.
/// If these components are discarded when the placeholder is despawned, all door/switch
/// interactions on the client become permanently broken.
///
/// ## Why visual components are optional
///
/// Local placeholders begin at `HydrationStage<1>`. `Mesh2d` and `MeshMaterial2d` are not
/// added until later hydration stages. If the server entity arrives before hydration
/// completes, the mesh query fails and `Added<TmxEntityId>` expires — the stitch window
/// is lost forever. Making mesh and material optional allows the stitch to proceed with
/// logic components regardless of hydration progress; the visuals will be absent until the
/// local placeholder finishes hydrating, at which point the stitcher will not re-fire
/// (because `Added<TmxEntityId>` has already been consumed). This is an acceptable
/// trade-off: a briefly invisible door is better than a permanently non-interactive one.
/// In practice the server entity arrives after the client has had time to hydrate, so
/// this race should be rare.
fn stitch_map_entities(
    mut commands: Commands,
    q_server: Query<(Entity, &TmxEntityId), (Added<TmxEntityId>, With<Replicated>)>,
    q_local: Query<
        (
            Entity,
            &TmxEntityId,
            &Transform,
            &Position,
            &Visibility,
            &Behavior,
            Option<&Interactive>,
            Option<&RoomState>,
            Option<&MapEntityFieldBPos>,
            Option<&MeshMaterial2d<CustomMaterial1>>,
            Option<&Mesh2d>,
        ),
        Without<Replicated>,
    >,
) {
    for (server_ent, server_id) in q_server.iter() {
        // Find the local placeholder that matches this replicated entity.
        let found = q_local
            .iter()
            .find(|(_, local_id, ..)| *local_id == server_id);

        let Some((
            local_ent,
            _,
            transform,
            position,
            visibility,
            behavior,
            interactive,
            room_state,
            map_bpos,
            material,
            mesh,
        )) = found
        else {
            warn!(
                "stitch_map_entities: replicated entity {:?} has TmxEntityId {:?} but no \
                 matching local placeholder was found. Skipping.",
                server_ent, server_id
            );
            continue;
        };

        // Build the insert bundle for the server entity. Start with components
        // that are always present on a hydrated placeholder.
        let mut ec = commands.entity(server_ent);
        ec.insert((
            *transform,
            *position,
            *visibility,
            behavior.clone(),
            GameSprite, // ensure cleanup marker is present on mission exit
        ));

        // Transfer optional logic components.
        if let Some(i) = interactive {
            ec.insert(i.clone());
        }
        if let Some(rs) = room_state {
            ec.insert(rs.clone());
        }
        if let Some(bpos) = map_bpos {
            ec.insert(bpos.clone());
        }

        // Transfer optional visual components (may be absent if hydration is not yet done).
        if let Some(mat) = material {
            ec.insert(mat.clone());
        }
        if let Some(m) = mesh {
            ec.insert(m.clone());
        }

        // Despawn the now-redundant placeholder.
        commands.entity(local_ent).despawn();

        debug!(
            "stitch_map_entities: stitched local {:?} → server {:?} (id={:?})",
            local_ent, server_ent, server_id
        );
    }
}
