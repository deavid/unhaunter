/// Smooths audio volume transitions using a cubic curve approach.
///
/// Implements smooth volume transitions for both fade-in and fade-out using a perceptual
/// curve (cubic root) that feels natural and avoids mathematical singularities at zero.
///
/// # Arguments
/// * `current_linear` - Current linear volume (0.0..1.0 or higher for overdrive)
/// * `target_linear` - Desired linear volume endpoint
/// * `speed` - Rate of change in perceptual units per second (e.g., 10.0 units/s for a 100ms fade)
/// * `dt_secs` - Delta time in seconds for this frame
///
/// # Returns
/// New linear volume after smoothing
pub fn smooth_volume(current_linear: f32, target_linear: f32, speed: f32, dt_secs: f32) -> f32 {
    // Clamp to non-negative numbers
    let current_linear = current_linear.max(0.0);
    let target_linear = target_linear.max(0.0);

    // Convert to perceptual space: perc = cubic_root(linear)
    let current_perc = current_linear.cbrt();
    let target_perc = target_linear.cbrt();

    // Calculate how much we can move per frame
    let max_change = speed * dt_secs;
    let diff = target_perc - current_perc;

    // Move current towards target by at most max_change
    let new_perc = if diff.abs() < max_change {
        target_perc
    } else {
        current_perc + diff.signum() * max_change
    };

    // Convert back to linear: linear = perc ^ 3
    new_perc.max(0.0).powi(3)
}
