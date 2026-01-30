use bevy::prelude::*;

#[derive(Resource, Debug, Reflect)]
pub struct FpsLimit(pub f32);

impl Default for FpsLimit {
    fn default() -> Self {
        Self(60.0)
    }
}

#[derive(Resource, Debug, Reflect, Default)]
pub struct FpsLimitRemaining(pub f32);

#[derive(Resource, Debug, Reflect, Default)]
pub struct FpsLimitUsage(pub f32);
