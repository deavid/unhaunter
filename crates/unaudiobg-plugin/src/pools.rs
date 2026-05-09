use bevy::ecs::system::Commands;
use bevy::prelude::SpawnRelated;
use bevy_seedling::prelude::PoolLabel;
use bevy_seedling::prelude::SamplerPool;
use bevy_seedling::prelude::VolumeNode;
use bevy_seedling::prelude::sample_effects;

#[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct BGAudioPool;

pub(crate) fn create_pools(mut commands: Commands) {
    commands.spawn((
        SamplerPool(BGAudioPool),
        sample_effects![VolumeNode {
            volume: bevy_seedling::prelude::Volume::Decibels(-120.0),
            ..Default::default()
        }],
    ));
}
