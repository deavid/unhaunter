use clap::{Parser, Subcommand, ValueEnum};
use enum_iterator::all;
use uncore::types::ghost::types::GhostType;

use crate::analysis::show_stats;
use crate::analysis::{
    handle_conflicts_command, handle_correlation_command, handle_unique_combinations_command,
}; // Added new analysis handlers
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
    /// Find optimal ghost sets based on target evidence and size
    FindSets {
        #[arg(long, help = "Comma-separated list of target evidences for the set")]
        target_evidence: String,
        #[arg(long, help = "Desired number of ghosts in the set")]
        size: usize,
        #[arg(
            long,
            help = "Maximum number of top sets to display",
            default_value = "5"
        )]
        max_results: usize,
    },
    /// Analyze evidence conflicts and overlaps
    Conflicts {
        #[arg(long, help = "Specific evidence subset to analyze (comma-separated)")]
        evidence: Option<String>,
        #[arg(long, help = "Show all types of conflicts", default_value = "false")]
        show_all: bool,
    },
    /// Show unique evidence combinations for identifying ghosts
    UniqueCombinations {
        #[arg(long, help = "Minimum number of evidences in a combination")]
        min_evidence: Option<usize>,
        #[arg(long, help = "Maximum number of evidences in a combination")]
        max_evidence: Option<usize>,
    },
    /// Analyze correlation between evidences
    Correlate {
        #[arg(long, help = "Primary evidence to correlate (name)")]
        evidence: String,
        #[arg(long, help = "Secondary evidence to correlate with (name, optional)")]
        with: Option<String>,
    },
    /// Generate a balanced set of ghosts
    OptimizeSet {
        #[arg(long, help = "Desired number of ghosts in the set")]
        size: usize,
        #[arg(long, help = "How much to weigh evidence balance (e.g., 0.0 to 1.0)")]
        balance_factor: Option<f32>, // Optional, can use default in handler
        #[arg(
            long,
            help = "Maximum allowed overlap (e.g., max evidences shared between any two ghosts)"
        )]
        max_overlap: Option<usize>, // Optional, can use default in handler
    },
    /// Find sets that maximize evidence diversity
    DiverseSet {
        #[arg(long, help = "Desired number of ghosts in the set")]
        size: usize,
        #[arg(long, help = "Minimum number of unique evidences the set must cover")]
        min_evidence_coverage: Option<usize>, // Optional
    },
    /// Generate sets suitable for tutorials or specific gameplay scenarios
    TutorialSet {
        #[arg(long, help = "Desired number of ghosts in the set")]
        size: usize,
        #[arg(
            long,
            help = "Optimize for beginner-friendliness",
            default_value = "true"
        )]
        beginner_friendly: bool,
    },
    /// Compare multiple ghost sets (e.g., "Set1Name:GhostA,GhostB" "Set2Name:GhostC,GhostD")
    CompareSets {
        #[arg(help = "Two or more named ghost sets to compare")]
        sets: Vec<String>,
    },
    /// Analyze overlap between multiple ghost sets
    OverlapAnalysis {
        #[arg(long, help = "Two or more named ghost sets (e.g., \"Set1:GA,GB\")", value_name = "SET_SPEC", num_args = 2..)]
        sets: Vec<String>,
    },
    /// Merge multiple ghost sets into one
    MergeSets {
        #[arg(long, help = "Two or more named ghost sets to merge (e.g., \"Set1:GA,GB\")", value_name = "SET_SPEC", num_args = 2..)]
        sets: Vec<String>,
        #[arg(long, help = "Optimize the merged set after creation")]
        optimize: bool,
    },
    /// Show differences between two specific ghost sets
    DiffSets {
        #[arg(help = "Old set specification (e.g., \"OldSet:GhostA,GhostB\")")]
        old_set: String,
        #[arg(help = "New set specification (e.g., \"NewSet:GhostA,GhostC\")")]
        new_set: String,
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
            Some(Commands::FindSets {
                target_evidence,
                size,
                max_results,
            }) => {
                // optimization::handle_find_sets_command(target_evidence, *size, *max_results);
                eprintln!("FindSets command is temporarily disabled for testing.");
            }
            Some(Commands::Conflicts { evidence, show_all }) => {
                handle_conflicts_command(evidence.as_deref(), *show_all);
            }
            Some(Commands::UniqueCombinations {
                min_evidence,
                max_evidence,
            }) => {
                handle_unique_combinations_command(*min_evidence, *max_evidence);
            }
            Some(Commands::Correlate { evidence, with }) => {
                handle_correlation_command(evidence, with.as_deref());
            }
            Some(Commands::OptimizeSet {
                size,
                balance_factor,
                max_overlap,
            }) => {
                // optimization::handle_optimize_set_command(*size, *balance_factor, *max_overlap);
                eprintln!("OptimizeSet command is temporarily disabled for testing.");
            }
            Some(Commands::DiverseSet {
                size,
                min_evidence_coverage,
            }) => {
                // optimization::handle_diverse_set_command(*size, *min_evidence_coverage);
                eprintln!("DiverseSet command is temporarily disabled for testing.");
            }
            Some(Commands::TutorialSet {
                size,
                beginner_friendly,
            }) => {
                // optimization::handle_tutorial_set_command(*size, *beginner_friendly);
                eprintln!("TutorialSet command is temporarily disabled for testing.");
            }
            Some(Commands::CompareSets { sets }) => {
                // comparison::handle_compare_sets_command(sets.clone());
                eprintln!("CompareSets command is temporarily disabled for testing.");
            }
            Some(Commands::OverlapAnalysis { sets }) => {
                // comparison::handle_overlap_analysis_command(sets.clone());
                eprintln!("OverlapAnalysis command is temporarily disabled for testing.");
            }
            Some(Commands::MergeSets { sets, optimize }) => {
                // comparison::handle_merge_sets_command(sets.clone(), *optimize);
                eprintln!("MergeSets command is temporarily disabled for testing.");
            }
            Some(Commands::DiffSets { old_set, new_set }) => {
                // comparison::handle_diff_sets_command(old_set.clone(), new_set.clone());
                eprintln!("DiffSets command is temporarily disabled for testing.");
            }
            None => show_ghost_list(&ghosts, &self.format),
        }
    }
}
