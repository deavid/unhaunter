use bevy::prelude::*;

#[derive(Debug, Component, Clone)]
pub(crate) struct SCamera;

#[derive(Debug, Component, Clone)]
pub(crate) struct SummaryUI;

#[derive(Debug, Component, Clone)]
pub(crate) enum SummaryUIType {
    GhostList,
    TimeTaken,
    MapMissionName,
    DifficultyName,
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
