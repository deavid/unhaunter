use bevy::prelude::*;

#[derive(Resource, Debug, Reflect)]
pub(crate) struct FpsLimit(pub f32);

impl Default for FpsLimit {
    fn default() -> Self {
        Self(60.0)
    }
}

#[derive(Resource, Debug, Reflect, Default)]
pub(crate) struct FpsLimitRemaining(pub f32);

#[derive(Resource, Debug, Reflect, Default)]
pub(crate) struct FpsLimitUsage(pub f32);
