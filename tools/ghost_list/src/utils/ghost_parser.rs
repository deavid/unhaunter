use enum_iterator::all;
use uncore::types::ghost::types::GhostType;

pub fn parse_ghost_list(ghost_str: &str) -> Vec<GhostType> {
    ghost_str
        .split(',')
        .map(|s| s.trim())
        .filter_map(|name| {
            // Find ghost by name (case insensitive)
            all::<GhostType>().find(|ghost| ghost.name().eq_ignore_ascii_case(name))
        })
        .collect()
}
