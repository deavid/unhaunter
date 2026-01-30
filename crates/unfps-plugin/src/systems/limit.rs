use crate::metrics::{LIMIT_REMAINING, LIMIT_USAGE};
use bevy::prelude::*;
use std::time::Instant;
use unfps_core::resources::{FpsLimit, FpsLimitRemaining, FpsLimitUsage};
use unmetrics_core::metrics::SendMetric;

#[cfg(not(target_arch = "wasm32"))]
use std::thread::sleep;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
struct LastFrameEnd(Instant);

pub(crate) fn app_setup(app: &mut App) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.insert_resource(LastFrameEnd(Instant::now()));
        app.add_systems(Last, fps_limiter);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn fps_limiter(
    fps_limit: Res<FpsLimit>,
    mut last_frame_end: ResMut<LastFrameEnd>,
    mut remaining_res: ResMut<FpsLimitRemaining>,
    mut usage_res: ResMut<FpsLimitUsage>,
) {
    let target_fps = fps_limit.0;
    if target_fps <= 0.0 {
        last_frame_end.0 = Instant::now();
        remaining_res.0 = 0.0;
        usage_res.0 = 0.0;
        LIMIT_REMAINING.tx(0.0);
        LIMIT_USAGE.tx(0.0);
        return;
    }

    let target_frame_time = std::time::Duration::from_secs_f32(1.0 / target_fps);
    let now = Instant::now();
    let elapsed = now.duration_since(last_frame_end.0);

    let remaining = target_frame_time.as_secs_f32() - elapsed.as_secs_f32();
    let usage = elapsed.as_secs_f32() / target_frame_time.as_secs_f32();
    remaining_res.0 = remaining;
    usage_res.0 = usage;
    LIMIT_REMAINING.tx(remaining as f64 * 1000.0);
    LIMIT_USAGE.tx(usage as f64 * 100.0);

    if elapsed < target_frame_time {
        let remaining_duration = target_frame_time - elapsed;
        sleep(remaining_duration);
    }
    last_frame_end.0 = Instant::now();
}
