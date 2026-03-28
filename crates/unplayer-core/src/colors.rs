use bevy::color::{Color, Hsla};

pub fn player_color(index: usize) -> Color {
    if index >= 9 {
        return Color::WHITE;
    }
    let i = index as f32;
    // Math: Hue(i) = (23 + (i * 3 + (i / 3)) * 40) % 360
    // Simplified for 9 players with stride 3:
    // 0: 23, 1: 143, 2: 263
    // 3: 63, 4: 183, 5: 303
    // 6: 103, 7: 223, 8: 343
    let hue = (23.0 + (i * 120.0 + (i / 3.0).floor() * 40.0)) % 360.0;
    Color::Hsla(Hsla::new(hue, 0.8, 0.6, 1.0))
}
