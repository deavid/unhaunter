/// Smooths audio volume transitions using a unified decibel-based approach.
///
/// Implements smooth volume transitions for both fade-in and fade-out using an exponential
/// envelope that respects perceived loudness (decibels).
///
/// # Arguments
/// * `current_linear` - Current linear volume (0.0..1.0 or higher for overdrive)
/// * `target_linear` - Desired linear volume endpoint
/// * `db_per_second` - Rate of change in decibels per second (e.g., 10.0 dB/s)
/// * `dt_secs` - Delta time in seconds for this frame
///
/// # Returns
/// New linear volume after smoothing
pub fn smooth_volume_db(
    current_linear: f32,
    target_linear: f32,
    db_per_second: f32,
    dt_secs: f32,
) -> f32 {
    // Clamp to avoid log of zero
    let current_linear = current_linear.max(0.000001);
    let target_linear = target_linear.max(0.000001);

    // Convert to decibels: dB = 20 * log10(linear)
    let current_db = 20.0 * current_linear.log10();
    let target_db = 20.0 * target_linear.log10();

    // Calculate how much we can move per frame
    let max_db_change = db_per_second * dt_secs;

    // Move towards target by at most max_db_change
    let new_db = if (target_db - current_db).abs() < max_db_change {
        target_db
    } else if target_db > current_db {
        current_db + max_db_change
    } else {
        current_db - max_db_change
    };

    // Convert back to linear: linear = 10^(dB/20)
    10f32.powf(new_db / 20.0)
}
