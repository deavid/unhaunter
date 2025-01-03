use bevy::prelude::*;

#[derive(Debug, Component, Clone)]
pub struct SCamera;
#[derive(Debug, Component, Clone)]
pub struct SummaryUI;

#[derive(Debug, Component, Clone)]
pub enum SummaryUIType {
    GhostList,
    TimeTaken,
    GhostUnhaunted,
    RepellentUsed,
    AvgSanity,
    PlayersAlive,
    FinalScore,
}
