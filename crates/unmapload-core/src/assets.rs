use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_seedling::prelude::*;

#[derive(AssetCollection, Resource, Debug, Clone)]
pub struct MapAssets {
    #[asset(path = "img/vignette.png")]
    pub vignette: Handle<Image>,
}

#[derive(AssetCollection, Resource, Debug, Clone)]
pub struct MissionAssets {
    // Mission audio (keep alive to avoid in-mission load stutter).
    #[asset(path = "sounds/background-noise-house-1.ogg")]
    pub background_noise_house: Handle<AudioSample>,
    #[asset(path = "sounds/ambient-clean.ogg")]
    pub ambient_clean: Handle<AudioSample>,
    #[asset(path = "sounds/heartbeat-1.ogg")]
    pub heartbeat: Handle<AudioSample>,
    #[asset(path = "sounds/heartbeat-2.ogg")]
    pub heartbeat_2: Handle<AudioSample>,
    #[asset(path = "sounds/insane-1.ogg")]
    pub insane: Handle<AudioSample>,

    // Short SFX (<= 100 KB) used during missions.
    #[asset(path = "sounds/breaker_trip.ogg")]
    pub breaker_trip: Handle<AudioSample>,
    #[asset(path = "sounds/door-close.ogg")]
    pub door_close: Handle<AudioSample>,
    #[asset(path = "sounds/door_creak_slow.ogg")]
    pub door_creak_slow: Handle<AudioSample>,
    #[asset(path = "sounds/door-open.ogg")]
    pub door_open: Handle<AudioSample>,
    #[asset(path = "sounds/fadein-progress-1000ms.ogg")]
    pub fadein_progress: Handle<AudioSample>,
    #[asset(path = "sounds/fadein-progress-1500ms.ogg")]
    pub fadein_progress_1500: Handle<AudioSample>,
    #[asset(path = "sounds/effects-chirp-base.ogg")]
    pub effects_chirp_base: Handle<AudioSample>,
    #[asset(path = "sounds/effects-chirp-click.ogg")]
    pub effects_chirp_click: Handle<AudioSample>,
    #[asset(path = "sounds/effects-chirp-high.ogg")]
    pub effects_chirp_high: Handle<AudioSample>,
    #[asset(path = "sounds/effects-chirp-short.ogg")]
    pub effects_chirp_short: Handle<AudioSample>,
    #[asset(path = "sounds/effects-chirp-shorter.ogg")]
    pub effects_chirp_shorter: Handle<AudioSample>,
    #[asset(path = "sounds/effects-dingdingding.ogg")]
    pub effects_dingdingding: Handle<AudioSample>,
    #[asset(path = "sounds/effects-dongdongdong.ogg")]
    pub effects_dongdongdong: Handle<AudioSample>,
    #[asset(path = "sounds/effects-radio-answer1.ogg")]
    pub effects_radio_answer1: Handle<AudioSample>,
    #[asset(path = "sounds/effects-radio-answer2.ogg")]
    pub effects_radio_answer2: Handle<AudioSample>,
    #[asset(path = "sounds/effects-radio-answer3.ogg")]
    pub effects_radio_answer3: Handle<AudioSample>,
    #[asset(path = "sounds/effects-radio-answer4.ogg")]
    pub effects_radio_answer4: Handle<AudioSample>,
    #[asset(path = "sounds/effects-radio-scan.ogg")]
    pub effects_radio_scan: Handle<AudioSample>,
    #[asset(path = "sounds/ghost-effect-1.ogg")]
    pub ghost_effect_1: Handle<AudioSample>,
    #[asset(path = "sounds/ghost-effect-2.ogg")]
    pub ghost_effect_2: Handle<AudioSample>,
    #[asset(path = "sounds/ghost-effect-3.ogg")]
    pub ghost_effect_3: Handle<AudioSample>,
    #[asset(path = "sounds/ghost-effect-4.ogg")]
    pub ghost_effect_4: Handle<AudioSample>,
    #[asset(path = "sounds/ghost-roar-1.ogg")]
    pub ghost_roar_1: Handle<AudioSample>,
    #[asset(path = "sounds/ghost-roar-2.ogg")]
    pub ghost_roar_2: Handle<AudioSample>,
    #[asset(path = "sounds/ghost-roar-3.ogg")]
    pub ghost_roar_3: Handle<AudioSample>,
    #[asset(path = "sounds/ghost-roar-4.ogg")]
    pub ghost_roar_4: Handle<AudioSample>,
    #[asset(path = "sounds/ghost-snore-1.ogg")]
    pub ghost_snore_1: Handle<AudioSample>,
    #[asset(path = "sounds/ghost-snore-2.ogg")]
    pub ghost_snore_2: Handle<AudioSample>,
    #[asset(path = "sounds/ghost-snore-3.ogg")]
    pub ghost_snore_3: Handle<AudioSample>,
    #[asset(path = "sounds/ghost-snore-4.ogg")]
    pub ghost_snore_4: Handle<AudioSample>,
    #[asset(path = "sounds/hide-rustle.ogg")]
    pub hide_rustle: Handle<AudioSample>,
    #[asset(path = "sounds/impact_glass_shatter.ogg")]
    pub impact_glass_shatter: Handle<AudioSample>,
    #[asset(path = "sounds/impact_metal_clink.ogg")]
    pub impact_metal_clink: Handle<AudioSample>,
    #[asset(path = "sounds/impact_wood_thud.ogg")]
    pub impact_wood_thud: Handle<AudioSample>,
    #[asset(path = "sounds/invalid-action-buzz.ogg")]
    pub invalid_action_buzz: Handle<AudioSample>,
    #[asset(path = "sounds/item-drop-clunk.ogg")]
    pub item_drop_clunk: Handle<AudioSample>,
    #[asset(path = "sounds/item-drop-clunk-1.ogg")]
    pub item_drop_clunk_1: Handle<AudioSample>,
    #[asset(path = "sounds/item-move-scrape.ogg")]
    pub item_move_scrape: Handle<AudioSample>,
    #[asset(path = "sounds/item-pickup-whoosh.ogg")]
    pub item_pickup_whoosh: Handle<AudioSample>,
    #[asset(path = "sounds/object_drag_wood.ogg")]
    pub object_drag_wood: Handle<AudioSample>,
    #[asset(path = "sounds/object_nudge_1.ogg")]
    pub object_nudge_1: Handle<AudioSample>,
    #[asset(path = "sounds/object_throw_generic.ogg")]
    pub object_throw_generic: Handle<AudioSample>,
    #[asset(path = "sounds/quartz_crack.ogg")]
    pub quartz_crack: Handle<AudioSample>,
    #[asset(path = "sounds/radio-off-zzt.ogg")]
    pub radio_off_zzt: Handle<AudioSample>,
    #[asset(path = "sounds/radio-on-zzt.ogg")]
    pub radio_on_zzt: Handle<AudioSample>,
    #[asset(path = "sounds/sage_activation.ogg")]
    pub sage_activation: Handle<AudioSample>,
    #[asset(path = "sounds/salt_drop.ogg")]
    pub salt_drop: Handle<AudioSample>,
    #[asset(path = "sounds/door_lock_heavy.ogg")]
    pub door_lock_heavy: Handle<AudioSample>,
    #[asset(path = "sounds/switch-off-1.ogg")]
    pub switch_off_1: Handle<AudioSample>,
    #[asset(path = "sounds/switch-on-1.ogg")]
    pub switch_on_1: Handle<AudioSample>,
    #[asset(path = "sounds/switch-on-2.ogg")]
    pub switch_on: Handle<AudioSample>,

    // Mission visuals (particles/overlays spawned as transient entities).
    #[asset(path = "img/smoke.png")]
    pub smoke: Handle<Image>,
    #[asset(path = "img/salt_pile.png")]
    pub salt_pile: Handle<Image>,
    #[asset(path = "img/salt_particle.png")]
    pub salt_particle: Handle<Image>,
    #[asset(path = "img/particle_small.png")]
    pub particle_small: Handle<Image>,
    #[asset(path = "img/particle_dust.png")]
    pub particle_dust: Handle<Image>,
    #[asset(path = "img/particle_glow.png")]
    pub particle_glow: Handle<Image>,
    #[asset(path = "img/particle_spark.png")]
    pub particle_spark: Handle<Image>,
    #[asset(path = "img/hiding_overlay.png")]
    pub hiding_overlay: Handle<Image>,
}

pub const GRID_1X1_ANCHOR: Vec2 = Vec2::new(0.0, -0.2045455); // calc(18, 31, 36, 44)
pub const GRID_1X1X4_ANCHOR: Vec2 = Vec2::new(0.0, -0.3673469); // calc(18, 85, 36, 98)
