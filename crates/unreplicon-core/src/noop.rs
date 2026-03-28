use bevy::prelude::*;
use bevy_replicon::bytes::Bytes;
use bevy_replicon::shared::replication::deferred_entity::DeferredEntity;
use bevy_replicon::shared::replication::registry::ctx::{RemoveCtx, WriteCtx};
use bevy_replicon::shared::replication::registry::rule_fns::RuleFns;

/// Discards the server's component value without writing it to the entity.
/// Use this as the `write` function in `app.set_marker_fns` for components
/// that a locally-owned client controls authoritatively.
pub fn noop_write<C: Component>(
    ctx: &mut WriteCtx,
    rule_fns: &RuleFns<C>,
    _entity: &mut DeferredEntity,
    message: &mut Bytes,
) -> Result<(), bevy::prelude::BevyError> {
    // Deserialize and discard — advances the message cursor correctly.
    let _ = rule_fns.deserialize(ctx, message)?;
    Ok(())
}

/// Suppresses the server's component removal on a locally-owned entity.
/// Use this as the `remove` function in `app.set_marker_fns`.
pub fn noop_remove(_ctx: &mut RemoveCtx, _entity: &mut DeferredEntity) {
    // Intentionally empty.
}
