use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct EvidenceUI;

#[derive(Component, Debug)]
pub struct GameUI;

#[derive(Component, Debug, PartialEq, Eq)]
pub enum ElementObjectUI {
    Name,
    Description,
    Grab,
}

#[derive(Component, Debug)]
pub struct DamageBackground {
    pub exp: f32,
}

impl DamageBackground {
    pub fn new(exp: f32) -> Self {
        Self { exp }
    }
}
#[derive(Component, Debug)]
pub struct HeldObjectUI;
#[derive(Component, Debug)]
pub struct RightSideGearUI;

#[derive(Component, Debug, Default)]
pub struct WalkieTextUIRoot;

#[derive(Component, Debug)]
pub struct WalkieText;
