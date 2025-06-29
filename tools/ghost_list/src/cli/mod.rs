use clap::{Parser, Subcommand, ValueEnum};
use enum_iterator::all;
use uncore::types::ghost::types::GhostType;

use crate::analysis::show_stats;
use crate::export::show_ghost_list;
use crate::filtering::apply_evidence_filters;
use crate::sets::{analyze_set, complete_set, test_set, validate_set};

#[derive(Parser)]
#[command(name = "ghost_list")]
#[command(about = "Unhaunter ghost analysis and set optimization tool")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    // Global filtering options
    #[arg(
        long,
        help = "Filter ghosts that have ALL specified evidence (comma-separated)"
    )]
    pub has_evidence: Option<String>,

    #[arg(
        long,
        help = "Filter ghosts that are missing ALL specified evidence (comma-separated)"
    )]
    pub missing_evidence: Option<String>,

    #[arg(
        long,
        help = "Filter ghosts that have ALL specified evidence (comma-separated)"
    )]
    pub has_all: Option<String>,

    #[arg(
        long,
        help = "Filter ghosts that have ANY of the specified evidence (comma-separated)"
    )]
    pub has_any: Option<String>,

    // Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show evidence statistics and distribution
    Stats,
    /// Test a ghost set for balance and uniqueness
    TestSet {
        #[arg(help = "Comma-separated list of ghost names")]
        ghosts: String,
    },
    /// Analyze a ghost set for balance and gaps
    AnalyzeSet {
        #[arg(help = "Comma-separated list of ghost names")]
        ghosts: String,
    },
    /// Find ghosts that would complete a partial set
    CompleteSet {
        #[arg(help = "Comma-separated list of existing ghost names")]
        ghosts: String,
        #[arg(
            long,
            help = "Required evidence that candidates must have (comma-separated)"
        )]
        requires_evidence: Option<String>,
        #[arg(
            long,
            help = "Evidence that candidates must NOT have (comma-separated)"
        )]
        excludes_evidence: Option<String>,
        #[arg(
            long,
            help = "Maximum number of candidates to show",
            default_value = "10"
        )]
        max_candidates: usize,
    },
    /// Validate if a set is uniquely identifiable
    ValidateSet {
        #[arg(help = "Comma-separated list of ghost names")]
        ghosts: String,
        #[arg(
            long,
            help = "Minimum evidence needed for unique identification",
            default_value = "2"
        )]
        min_evidence: usize,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

impl Cli {
    pub fn execute(&self) {
        // Get all ghosts and apply global filters
        let mut ghosts: Vec<GhostType> = all::<GhostType>().collect();

        // Apply evidence filters
        ghosts = apply_evidence_filters(ghosts, self);

        // Sort alphabetically
        ghosts.sort_by(|a, b| a.name().cmp(b.name()));

        match &self.command {
            Some(Commands::Stats) => show_stats(&ghosts, &self.format),
            Some(Commands::TestSet {
                ghosts: ghost_names,
            }) => test_set(ghost_names),
            Some(Commands::AnalyzeSet {
                ghosts: ghost_names,
            }) => analyze_set(ghost_names),
            Some(Commands::CompleteSet {
                ghosts: ghost_names,
                requires_evidence,
                excludes_evidence,
                max_candidates,
            }) => complete_set(
                ghost_names,
                requires_evidence.as_deref(),
                excludes_evidence.as_deref(),
                *max_candidates,
            ),
            Some(Commands::ValidateSet {
                ghosts: ghost_names,
                min_evidence,
            }) => validate_set(ghost_names, *min_evidence),
            None => show_ghost_list(&ghosts, &self.format),
        }
    }
}
