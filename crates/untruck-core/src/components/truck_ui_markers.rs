use bevy::prelude::*;

/// Marker component for the root entity of the Truck UI panel.
#[derive(Component, Debug)]
pub struct TruckUI;

/// Marker component for the text node that displays the current ghost guess.
#[derive(Component, Debug)]
pub struct TruckUIGhostGuess;
