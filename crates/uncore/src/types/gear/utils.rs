/// Utility function to convert a boolean value to "ON" or "OFF".
pub fn on_off(s: bool) -> &'static str {
    match s {
        true => "ON",
        false => "OFF",
    }
}
