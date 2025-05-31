use bevy::prelude::*;

#[derive(Debug, Component, Clone)]
pub struct SCamera;
#[derive(Debug, Component, Clone)]
pub struct SummaryUI;

#[derive(Debug, Component, Clone)]
pub enum SummaryUIType {
    GhostList,
    TimeTaken,
    MapMissionName, // Added
    DifficultyName, // Added
    GhostUnhaunted,
    RepellentUsed,
    AvgSanity,
    PlayersAlive,
    FinalScore,
    GradeAchieved,
    BaseReward,
    GradeMultiplier,
    CalculatedEarnings,
    InsuranceDepositHeld,
    CostsDeducted,
    DepositReturned,
    NetChange,
    FinalBankTotal,
}
