use bevy::{
    prelude::{Deref, DerefMut},
    time::{Timer, TimerMode},
};

pub fn format_time(time_in_seconds: f32) -> String {
    let minutes = (time_in_seconds / 60.0).floor() as u32;
    let seconds = (time_in_seconds % 60.0).floor() as u32;
    let hundredths = ((time_in_seconds % 60.0 - seconds as f32) * 100.0).floor() as u32;

    format!("{:02}m {:02}.{:02}s", minutes, seconds, hundredths) // 99:99.00 format
}

#[derive(Deref, DerefMut)]
pub struct PrintingTimer(Timer);

impl Default for PrintingTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(5.0, TimerMode::Repeating))
    }
}

#[derive(Default)]
pub struct MeanValue {
    pub mean: f32,
    pub len: f32,
}

impl MeanValue {
    pub fn _push(&mut self, val: f32) {
        self.push_len(val, 1.0)
    }
    pub fn push_len(&mut self, val: f32, len: f32) {
        if len > 0.0 {
            self.mean = (self.mean * self.len + val * len) / (self.len + len);
            self.len += len;
        }
    }
    pub fn avg(&mut self) -> f32 {
        self.len = 0.0000001;
        self.mean
    }
}
