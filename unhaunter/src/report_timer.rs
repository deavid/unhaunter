use std::time::Duration;

use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;

pub fn report_performance(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut timer: Local<ReportTimer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut system_times: Vec<(&str, f64)> = Vec::new();

        for diagnostic in diagnostics.iter() {
            if let Some(average) = diagnostic.average() {
                let system_name = diagnostic.path().as_str();
                system_times.push((system_name, average));
            }
        }

        info!("--- Performance Report ---");
        for (name, time) in system_times.iter() {
            info!("{}: {:.2}", name, time);
        }
        info!("------------------------");
    }
}

pub struct ReportTimer(Timer);

impl Default for ReportTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(10), TimerMode::Repeating))
    }
}
