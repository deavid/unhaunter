pub fn format_time(time_in_seconds: f32) -> String {
    let minutes = (time_in_seconds / 60.0).floor() as u32;
    let seconds = (time_in_seconds % 60.0).floor() as u32;
    let hundredths = ((time_in_seconds % 60.0 - seconds as f32) * 100.0).floor() as u32;

    format!("{:02}m {:02}.{:02}s", minutes, seconds, hundredths) // 99:99.00 format
}
