use bevy::prelude::Resource;

#[derive(Resource, Default, PartialEq, Eq)]
pub(crate) enum LevelLoadingStatus {
    #[default]
    Idle,
    JustStarted,
    InProgress,
    Complete,
}
