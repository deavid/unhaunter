use bevy::prelude::*;

/// Marker component for the root UI node of the on-screen hint.
#[derive(Component, Debug, Default)]
pub struct HintBoxUIRoot;

/// Marker component for the text UI node of the on-screen hint.
#[derive(Component, Debug, Default)]
pub struct HintBoxText;
