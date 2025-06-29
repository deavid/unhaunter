#[cfg(test)]
mod tests {
    use crate::events::{WalkieEvent, WalkieEventPriority, WalkieRepeatBehavior};

    #[test]
    fn test_effective_priority_downgrading() {
        // Test that a VeryLowRepeat event gets heavily downgraded after being played
        let event = WalkieEvent::GearExplanation(uncore::types::gear_kind::GearKind::Flashlight);

        // Initial priority should be VeryHigh
        assert_eq!(event.priority(), WalkieEventPriority::VeryHigh);
        assert_eq!(event.repeat_behavior(), WalkieRepeatBehavior::VeryLowRepeat);

        // After 0 plays, effective priority should be the same
        assert_eq!(event.effective_priority(0), WalkieEventPriority::VeryHigh);

        // After 1 play, should drop 2 levels: VeryHigh -> Medium
        assert_eq!(event.effective_priority(1), WalkieEventPriority::Medium);

        // After 2 plays, should drop 3 levels: VeryHigh -> Low
        assert_eq!(event.effective_priority(2), WalkieEventPriority::Low);

        // After 3+ plays, should drop to minimum: VeryHigh -> VeryLow
        assert_eq!(event.effective_priority(3), WalkieEventPriority::VeryLow);
        assert_eq!(event.effective_priority(10), WalkieEventPriority::VeryLow);
    }

    #[test]
    fn test_always_repeat_events_not_downgraded() {
        // Test that AlwaysRepeat events maintain their priority
        let event =
            WalkieEvent::IncorrectRepellentHint(uncore::types::evidence::Evidence::FreezingTemp);

        // Should be VeryHigh priority with AlwaysRepeat behavior
        assert_eq!(event.priority(), WalkieEventPriority::VeryHigh);
        assert_eq!(event.repeat_behavior(), WalkieRepeatBehavior::AlwaysRepeat);

        // Should maintain priority regardless of play count
        assert_eq!(event.effective_priority(0), WalkieEventPriority::VeryHigh);
        assert_eq!(event.effective_priority(1), WalkieEventPriority::VeryHigh);
        assert_eq!(event.effective_priority(5), WalkieEventPriority::VeryHigh);
        assert_eq!(event.effective_priority(100), WalkieEventPriority::VeryHigh);
    }

    #[test]
    fn test_normal_repeat_gradual_downgrade() {
        // Test that NormalRepeat events get gradually downgraded
        let event = WalkieEvent::PlayerStuckAtStart;

        // Should be Medium priority with NormalRepeat behavior
        assert_eq!(event.priority(), WalkieEventPriority::Medium);
        assert_eq!(event.repeat_behavior(), WalkieRepeatBehavior::NormalRepeat);

        // Should maintain priority for first few plays
        assert_eq!(event.effective_priority(0), WalkieEventPriority::Medium);
        assert_eq!(event.effective_priority(1), WalkieEventPriority::Medium);

        // Should drop 1 level after 2-4 plays: Medium -> Low
        assert_eq!(event.effective_priority(2), WalkieEventPriority::Low);
        assert_eq!(event.effective_priority(4), WalkieEventPriority::Low);

        // Should drop 2 levels after 5-9 plays: Medium -> VeryLow
        assert_eq!(event.effective_priority(5), WalkieEventPriority::VeryLow);
        assert_eq!(event.effective_priority(9), WalkieEventPriority::VeryLow);

        // Should maintain minimum after many plays
        assert_eq!(event.effective_priority(15), WalkieEventPriority::VeryLow);
    }

    #[test]
    fn test_high_repeat_minimal_downgrade() {
        // Test that HighRepeat events get minimally downgraded
        let event = WalkieEvent::FreezingTempsEvidenceConfirmed;

        // Should be VeryHigh priority with HighRepeat behavior
        assert_eq!(event.priority(), WalkieEventPriority::VeryHigh);
        assert_eq!(event.repeat_behavior(), WalkieRepeatBehavior::HighRepeat);

        // Should maintain priority for first several plays
        assert_eq!(event.effective_priority(0), WalkieEventPriority::VeryHigh);
        assert_eq!(event.effective_priority(3), WalkieEventPriority::VeryHigh);

        // Should drop only 1 level after 4-9 plays: VeryHigh -> High
        assert_eq!(event.effective_priority(4), WalkieEventPriority::High);
        assert_eq!(event.effective_priority(9), WalkieEventPriority::High);

        // Should drop only 2 levels after many plays: VeryHigh -> Medium
        assert_eq!(event.effective_priority(10), WalkieEventPriority::Medium);
        assert_eq!(event.effective_priority(20), WalkieEventPriority::Medium);
    }
}
