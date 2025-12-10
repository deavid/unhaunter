use enum_iterator::all;
use uncore::types::evidence::Evidence;

pub fn parse_evidence_list(evidence_str: &str) -> Vec<Evidence> {
    evidence_str
        .split(',')
        .map(|s| s.trim())
        .filter_map(|name| {
            // Find evidence by name
            all::<Evidence>().find(|evidence| evidence.name().eq_ignore_ascii_case(name))
        })
        .collect()
}
