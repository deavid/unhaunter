pub mod csv;
pub mod json;
pub mod table;

use crate::cli::OutputFormat;
use uncore::types::ghost::types::GhostType;

pub fn show_ghost_list(ghosts: &[GhostType], format: &OutputFormat) {
    match format {
        OutputFormat::Table => table::show_ghost_table(ghosts),
        OutputFormat::Json => json::show_ghost_json(ghosts),
        OutputFormat::Csv => csv::show_ghost_csv(ghosts),
    }
}
