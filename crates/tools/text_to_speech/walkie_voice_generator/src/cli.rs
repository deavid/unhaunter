//! CLI argument parsing for the walkie_voice_generator tool.

use clap::Parser;

/// Defines the command-line arguments for the `walkie_voice_generator` tool.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// If set, generates a sample RON file to stdout and exits.
    /// This is useful for users to see the expected input format.
    #[clap(long, help = "Generate a sample RON file to stdout")]
    pub generate_sample_ron: bool,

    /// If set, deletes unused OGG files from the generated assets directory.
    /// An OGG file is considered unused if it's present in the assets directory
    /// but not listed in the current manifest after processing all RON files.
    #[clap(
        long,
        help = "Delete unused OGG files from the generated assets directory"
    )]
    pub delete_unused: bool,

    /// Forces regeneration of audio for all lines or lines matching a specific conceptual ID pattern.
    /// - Use "all" to regenerate everything.
    /// - Use a string (e.g., "MyConcept") to regenerate lines whose conceptual ID contains that string.
    /// - Use a string ending with '*' (e.g., "MyPrefix*") to regenerate lines whose conceptual ID starts with that prefix.
    #[clap(
        long,
        help = "Force regeneration of audio for all or matching conceptual IDs (e.g., \'all\' or \'MyConceptPrefix*\')"
    )]
    pub force_regenerate: Option<String>,

    /// Specifies the number of parallel jobs for audio generation.
    #[clap(
        long,
        help = "Number of parallel jobs for audio generation",
        default_value_t = 6
    )]
    pub parallel_jobs: usize,
}
