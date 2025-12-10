def process_walkie_events():
    all_possible_ids = [
        'AllObjectivesMetReminderToEndMission', 'CPM500EvidenceConfirmed',
        'ClearEvidenceFoundNoActionCKey', 'ClearEvidenceFoundNoActionTruck',
        'DarkRoomNoLightUsed', 'DidNotCycleToOtherGear',
        'DidNotSwitchStartingGearInHotspot', 'DoorInteractionHesitation',
        'EMFLevel5EvidenceConfirmed', 'EMFMinorFluctuationsIgnored',
        'EMFNonEMF5Fixation', 'EVPEvidenceConfirmed', 'ErraticMovementEarly',
        'FlashlightOnInLitRoom', 'FloatingOrbsEvidenceConfirmed',
        'FreezingTempsEvidenceConfirmed', 'GearInVan', 'GearSelectedNotActivated',
        'GhostExpelledPlayerLingers', 'GhostExpelledPlayerMissed',
        'GhostNearHunt', 'HasRepellentEntersLocation',
        'HuntActiveNearHidingSpotNoHide', 'HuntWarningNoPlayerEvasion',
        'IgnoredObviousBreach', 'IgnoredVisibleGhost',
        'InTruckWithEvidenceNoJournal', 'JournalConflictingEvidence',
        'JournalPointsToOneGhostNoCraft', 'LowHealthGeneralWarning',
        'MissionStartEasy', 'PlayerLeavesTruckWithoutChangingLoadout',
        'PlayerStaysHiddenTooLong', 'PlayerStuckAtStart',
        'PotentialGhostIDWithNewEvidencePrompt', 'QuartzCrackedFeedback',
        'QuartzShatteredFeedback', 'QuartzUnusedInRelevantSituation',
        'RLPresenceEvidenceConfirmed', 'RepellentExhaustedGhostPresentCorrectType',
        'RepellentUsedGhostEnragesPlayerFlees', 'RepellentUsedTooFar',
        'RoomLightsOnGearNeedsDark', 'SageActivatedIneffectively',
        'SageUnusedDefensivelyDuringHunt', 'SageUnusedInRelevantSituation',
        'SanityDroppedBelowThresholdDarkness', 'SanityDroppedBelowThresholdGhost',
        'SpiritBoxEvidenceConfirmed', 'StrugglingWithGrabDrop',
        'StrugglingWithHideUnhide', 'ThermometerNonFreezingFixation',
        'UVEctoplasmEvidenceConfirmed', 'VeryLowSanityNoTruckReturn'
    ]

    player_profile_stats = {
        'DidNotSwitchStartingGearInHotspot': 1, 'CPM500EvidenceConfirmed': 2,
        'FreezingTempsEvidenceConfirmed': 3, 'GhostExpelledPlayerLingers': 2,
        'MissionStartEasy': 12, 'UVEctoplasmEvidenceConfirmed': 5,
        'DarkRoomNoLightUsed': 5, 'EMFLevel5EvidenceConfirmed': 4,
        'FloatingOrbsEvidenceConfirmed': 3, 'GhostNearHunt': 6,
        'SpiritBoxEvidenceConfirmed': 1, 'StrugglingWithHideUnhide': 2,
        'BreachShowcase': 7, 'GhostExpelledPlayerMissed': 4,
        'JournalPointsToOneGhostNoCraft': 4, 'RLPresenceEvidenceConfirmed': 3,
        'VeryLowSanityNoTruckReturn': 2, 'PlayerStaysHiddenTooLong': 1,
        'GearSelectedNotActivated': 7, 'GearInVan': 6, 'GhostShowcase': 7
    }

    all_ids_set = set(all_possible_ids)
    profile_ids_set = set(player_profile_stats.keys())

    missing_ids = sorted(list(all_ids_set - profile_ids_set))

    rare_events = []
    for conceptual_id, play_count in player_profile_stats.items():
        if play_count <= 2:
            rare_events.append({"conceptual_id": conceptual_id, "play_count": play_count})

    # Sort rare_events by conceptual_id for consistent output
    rare_events_sorted = sorted(rare_events, key=lambda x: x['conceptual_id'])

    # Prepare output for the subtask report
    # The problem asks for two lists:
    # - A list of strings for missing conceptual_id's.
    # - A list of dictionaries for rare conceptual_id's
    # The submit_subtask_report tool takes a string, so we'll format these lists as strings.

    output_str = f"Missing conceptual_ids: {str(missing_ids)}\n"
    output_str += f"Rare conceptual_ids: {str(rare_events_sorted)}"

    print(output_str)

if __name__ == "__main__":
    process_walkie_events()
