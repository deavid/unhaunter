// unwalkiecore/src/events.rs
use bevy::prelude::Event;

/// Sending this event will cause the walkie to play a message.
#[derive(Clone, Debug, Event, PartialEq, Eq, Hash)]
pub enum WalkieEvent {
    /// When the player forgets the stuff in the van.
    GearInVan,
    /// When the Ghost rage is near its limit.
    GhostNearHunt,
    /// Welcome message for easy difficulty.
    MissionStartEasy,
}

impl WalkieEvent {
    pub fn time_to_play(&self, count: u32) -> f64 {
        let count = count.max(1) as f64;
        match self {
            WalkieEvent::GearInVan => 120.0 * count,
            WalkieEvent::GhostNearHunt => 120.0 * count.cbrt(),

            // These should never be replayed:
            WalkieEvent::MissionStartEasy => 3600.0 * 24.0 * 7.0,
        }
    }
    /// Get the sound file for the event.
    pub fn sound_file_list(&self) -> Vec<&str> {
        match self {
            WalkieEvent::GearInVan => vec![
                "walkie/gear-in-van-1.ogg",
                "walkie/gear-in-van-2.ogg",
                "walkie/gear-in-van-3.ogg",
                "walkie/gear-in-van-4.ogg",
                "walkie/gear-in-van-5.ogg",
                "walkie/gear-in-van-6.ogg",
                "walkie/gear-in-van-7.ogg",
            ],
            WalkieEvent::GhostNearHunt => vec![
                "walkie/ghost-near-hunt-1.ogg",
                "walkie/ghost-near-hunt-2.ogg",
                "walkie/ghost-near-hunt-3.ogg",
                "walkie/ghost-near-hunt-4.ogg",
            ],
            WalkieEvent::MissionStartEasy => vec![
                "walkie/mission-start-easy-1.ogg",
                "walkie/mission-start-easy-2.ogg",
                "walkie/mission-start-easy-3.ogg",
                "walkie/mission-start-easy-4.ogg",
                "walkie/mission-start-easy-5.ogg",
                "walkie/mission-start-easy-6.ogg",
                "walkie/mission-start-easy-7.ogg",
                "walkie/mission-start-easy-8.ogg",
                "walkie/mission-start-easy-9.ogg",
            ],
        }
    }
    pub fn voice_text(sound_file: &str) -> &str {
        match sound_file {
            // Gear in van:
            "walkie/gear-in-van-1.ogg" => {
                "Your gear is in the van. Are you sure you want to go in without it?"
            }
            "walkie/gear-in-van-2.ogg" => {
                "Uh... Aren't you forgetting something? The van is full of gear. And your hands are empty."
            }
            "walkie/gear-in-van-3.ogg" => {
                "Wait a minute, you left your kit in the truck! You might want to go back and collect your gear."
            }
            "walkie/gear-in-van-4.ogg" => {
                "Wait! Where's your equipment? You need to grab it from the van before going in."
            }
            "walkie/gear-in-van-5.ogg" => {
                "Hold on a second! Did you remember to pick up your gear from the truck?"
            }
            "walkie/gear-in-van-6.ogg" => {
                "You might want to double-check the van. It seems like you're missing some essential tools."
            }
            "walkie/gear-in-van-7.ogg" => {
                "Don't forget to equip yourself from the van before heading inside!"
            }
            // Ghost near hunt:
            "walkie/ghost-near-hunt-1.ogg" => {
                "Activity levels are off the charts! Something is about to happen. You should get out of there."
            }
            "walkie/ghost-near-hunt-2.ogg" => {
                "The energy levels are surging... it's getting angry... I don't think you're safe there."
            }
            "walkie/ghost-near-hunt-3.ogg" => {
                "Static's spiking on my end... that's never good... you might want to leave before it's too late."
            }
            "walkie/ghost-near-hunt-4.ogg" => {
                "Uh... you're not alone in there. And whatever it is, it's not happy. I'd look for an exit path if I were you."
            }

            // Mission start easy:
            "walkie/mission-start-easy-1.ogg" => {
                "Alright, you're on site. Reports indicate significant paranormal activity. Standard procedure: locate, identify, and neutralize the entity."
            }
            "walkie/mission-start-easy-2.ogg" => {
                "Unhaunter, this is base. Seems you've arrived. We've got multiple reports of disturbances. Get in there; assess the situation, and deal with the problem."
            }
            "walkie/mission-start-easy-3.ogg" => {
                "Looks like you made it. This should be a good one to warm you up. Get inside and find what's causing all that ruckus."
            }
            "walkie/mission-start-easy-4.ogg" => {
                "Okay, you're at the location. Shouldn't be anything too crazy in there... just, you know, the usual ghostly stuff. Head in when you're ready."
            }
            "walkie/mission-start-easy-5.ogg" => {
                "Welcome to the job, Unhaunter. This is a pretty standard haunting, so it's a good place to start. Get inside and do your thing."
            }
            "walkie/mission-start-easy-6.ogg" => {
                "Alright... Rookie... This is it. Don't mess it up! Explore the location and take measurements. Find the ghost and expel it."
            }
            "walkie/mission-start-easy-7.ogg" => {
                "This is a good opportunity to practice, the entity should be easy to deal with. Use the thermometer to find the ghost room."
            }
            "walkie/mission-start-easy-8.ogg" => {
                "Base to Unhaunter, we are getting some readings here... Go inside and see what is that about."
            }
            "walkie/mission-start-easy-9.ogg" => {
                "Hello there. I'm picking up some faint activity, so it's not a total waste of time. See if you can find the source."
            }
            // Default:
            _ => "**BZZT**",
        }
    }
}
