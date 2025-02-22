use std::time::Duration;

use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;

pub fn report_performance(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut timer: Local<ReportTimer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut system_times: Vec<(&str, f64, String)> = Vec::new();

        for diagnostic in diagnostics.iter() {
            if let Some(average) = diagnostic.average() {
                let system_name = diagnostic.path().as_str();
                if system_name.starts_with("un") || system_name == "fps" {
                    system_times.push((system_name, average, diagnostic.suffix.to_string()));
                }
            }
        }
        // Grab the top-N
        system_times.sort_by_key(|x| ordered_float::OrderedFloat::from(-x.1));
        system_times.truncate(5);

        system_times.sort_by_key(|x| x.0);

        for (name, time, suffix) in system_times.iter() {
            info!("{name}: {time:.2} {suffix}");
        }
    }
}

pub struct ReportTimer(Timer);

impl Default for ReportTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(10), TimerMode::Repeating))
    }
}
