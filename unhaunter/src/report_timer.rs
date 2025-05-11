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
        system_times.truncate(15);

        system_times.sort_by_key(|x| x.0);

        let mut total_systems_time = 0.0;
        for (name, time, suffix) in system_times.iter() {
            if *time > 0.05 {
                info!("{name}: {time:.2} {suffix}");
            }
            if name.starts_with("un") && name.contains("/systems/") {
                total_systems_time += time;
            }
        }
        const MAX_TIME: f64 = 1000.0 / 60.0;
        info!("systems: {:.2}%", total_systems_time / MAX_TIME * 100.0);
    }
}

pub struct ReportTimer(Timer);

impl Default for ReportTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(60), TimerMode::Repeating))
    }
}
