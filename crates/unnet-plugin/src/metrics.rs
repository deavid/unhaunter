use bevy::{
    app::App,
    diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic},
};

pub(crate) const NETWORK_IO: DP = DP::const_new("unnet/systems/network_io");
pub(crate) const HOST_HANDLE_DISCONNECTS: DP =
    DP::const_new("unnet/systems/host_handle_disconnects");
pub(crate) const HOST_APPLY_INPUT: DP = DP::const_new("unnet/systems/host_apply_input");
pub(crate) const HANDSHAKE_HANDLER: DP = DP::const_new("unnet/systems/handshake_handler");
pub(crate) const HOST_SEND_SNAPSHOTS: DP = DP::const_new("unnet/systems/host_send_snapshots");
pub(crate) const CLIENT_SEND_INPUT: DP = DP::const_new("unnet/systems/client_send_input");
pub(crate) const CLIENT_APPLY_SNAPSHOTS: DP = DP::const_new("unnet/systems/client_apply_snapshots");
pub(crate) const CLIENT_CONNECTION_MONITOR: DP =
    DP::const_new("unnet/systems/client_connection_monitor");
pub(crate) const HOST_SEND_SUMMARY: DP = DP::const_new("unnet/systems/host_send_summary");
pub(crate) const AUTOSTART_NET_GAME: DP = DP::const_new("unnet/systems/autostart_net_game");
pub(crate) const CLIENT_PROCESS_PENDING_MAP: DP =
    DP::const_new("unnet/systems/client_process_pending_map");
pub(crate) const CLIENT_REQUEST_GRAB: DP = DP::const_new("unnet/systems/client_request_grab");
pub(crate) const STARTUP_NETWORK_SYSTEM: DP = DP::const_new("unnet/systems/startup_network_system");

pub(crate) fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(NETWORK_IO).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(HOST_HANDLE_DISCONNECTS).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(HOST_APPLY_INPUT).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(HANDSHAKE_HANDLER).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(HOST_SEND_SNAPSHOTS).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(CLIENT_SEND_INPUT).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(CLIENT_APPLY_SNAPSHOTS).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(CLIENT_CONNECTION_MONITOR).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(HOST_SEND_SUMMARY).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(AUTOSTART_NET_GAME).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(CLIENT_PROCESS_PENDING_MAP).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(CLIENT_REQUEST_GRAB).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(STARTUP_NETWORK_SYSTEM).with_suffix("ms"));
}
