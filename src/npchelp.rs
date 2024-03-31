use bevy::prelude::*;

use crate::behavior::component::NPCHelpDialog;

#[derive(Clone, Debug, Event)]
pub struct NPCHelpEvent {
    pub entity: Entity,
}

impl NPCHelpEvent {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

pub fn npchelp_event(mut ev_npc: EventReader<NPCHelpEvent>, npc: Query<(Entity, &NPCHelpDialog)>) {
    let Some(ev_npc) = ev_npc.read().next() else {
        return;
    };
    let Some(npcd) = npc
        .iter()
        .find(|(e, _)| *e == ev_npc.entity)
        .map(|(_, n)| n)
    else {
        warn!("Wrong entity for npchelp_event?");
        return;
    };
    warn!(npcd.dialog);
}

pub fn app_setup(app: &mut App) {
    app.add_event::<NPCHelpEvent>()
        .add_systems(Update, npchelp_event);
}
