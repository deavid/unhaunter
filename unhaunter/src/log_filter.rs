const UNHAUNTER_CRATES: &[&str] = &[
    "unhaunter",
    "unassets_core",
    "unbehavior",
    "unboard_core",
    "uncampaign_plugin",
    "unclassic_mode_plugin",
    "undifficulty_core",
    "undifficulty_plugin",
    "unengine_core",
    "unengine_plugin",
    "unevents_core",
    "unfog_core",
    "unfog_plugin",
    "unfoundation_core",
    "unfps_core",
    "unfps_plugin",
    "ungame_plugin",
    "ungear_core",
    "ungear_plugin",
    "ungearitems_core",
    "ungearitems_plugin",
    "unghost_core",
    "unghost_plugin",
    "unhub_client",
    "unhub_plugin",
    "uninteraction_core",
    "uninteraction_plugin",
    "uninvestigation_shared",
    "unlight_core",
    "unlight_plugin",
    "unlobby_plugin",
    "unmainmenu_plugin",
    "unmanual_core",
    "unmanual_plugin",
    "unmaphub_plugin",
    "unmapload_core",
    "unmapload_plugin",
    "unmenu_core",
    "unmenu_plugin",
    "unmenusettings_plugin",
    "unmetrics_core",
    "unmetrics_plugin",
    "unmission_plugin",
    "unnavigation_core",
    "unnoise_core",
    "unnpc_plugin",
    "unpicking_core",
    "unpicking_plugin",
    "unplayer_core",
    "unplayer_plugin",
    "unprofile_core",
    "unprofile_plugin",
    "unrender_core",
    "unrender_plugin",
    "unrender_std",
    "unreplicon_core",
    "unreplicon_plugin",
    "unroot_plugin",
    "unsettings_core",
    "unsettings_plugin",
    "unsound_core",
    "unsound_plugin",
    "unspatial_core",
    "unsummary_core",
    "unsummary_plugin",
    "untags_core",
    "unthermal_core",
    "unthermal_plugin",
    "untiled_core",
    "untmxmap_plugin",
    "untruck_core",
    "untruck_plugin",
    "untypes_core",
    "unui_core",
    "unui_plugin",
    "unwalkie_core",
    "unwalkie_plugin",
    "unwalkie_types",
];

/// Third-party networking crates that are worth watching when debugging multiplayer.
/// Capped at `debug` because their `trace` output is extremely high-volume.
const NETWORKING_CRATES: &[&str] = &[
    "bevy_replicon",
    "bevy_replicon_renet",
    "bevy_renet",
    "renet",
    "renet_netcode",
];

pub fn build_log_filter(verbose: u8) -> String {
    let level = match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    // Networking crates are capped at debug — their trace output is extremely noisy.
    let net_level = match verbose {
        0 => "warn",
        1 => "info",
        _ => "debug",
    };
    let mut filter = "warn,wgpu_hal=error".to_string();
    for c in UNHAUNTER_CRATES {
        filter.push_str(&format!(",{}={}", c, level));
    }
    for c in NETWORKING_CRATES {
        filter.push_str(&format!(",{}={}", c, net_level));
    }
    filter
}
