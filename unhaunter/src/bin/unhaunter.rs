use clap::Parser;
use std::str::FromStr;
use untypes_core::cli::CliOptions;
use untypes_core::difficulty::Difficulty;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, action)]
    draft_maps: bool,

    #[clap(long)]
    host: Option<u16>,

    #[clap(long)]
    bind: Vec<String>,

    #[clap(long)]
    join: Option<String>,

    #[clap(long)]
    map: Option<String>,

    #[clap(long)]
    difficulty: Option<String>,

    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[clap(long, action)]
    mute: bool,
}

fn main() {
    let args = Args::parse();

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Err(e) = unassets_core::assets::index_updater::update_assetidx_files() {
            eprintln!("Failed to update assetidx files: {}", e);
        }
    }

    let net_mode = if let Some(port) = args.host {
        let mut bind_addresses = args.bind.clone();
        if bind_addresses.is_empty() {
            bind_addresses.push("::".to_string());
            bind_addresses.push("0.0.0.0".to_string());
        }
        untypes_core::cli::NetMode::Host {
            port,
            bind_addresses,
        }
    } else if let Some(address) = args.join {
        untypes_core::cli::NetMode::Join { address }
    } else {
        untypes_core::cli::NetMode::Offline
    };

    // --- Validation ---
    let mut final_map_path = args.map.clone();

    if let Some(map_path) = &args.map {
        let path = std::path::Path::new(map_path);
        if path.exists() {
            if let Some(stripped) = map_path.strip_prefix("assets/") {
                final_map_path = Some(stripped.to_string());
            }
        } else {
            let alt_path_str = if map_path.starts_with("assets/") {
                map_path.clone()
            } else {
                format!("assets/{}", map_path)
            };

            let alt_path = std::path::Path::new(&alt_path_str);
            if alt_path.exists() {
                if let Some(stripped) = map_path.strip_prefix("assets/") {
                    final_map_path = Some(stripped.to_string());
                } else {
                    final_map_path = Some(map_path.clone());
                }
            } else {
                eprintln!("ERROR: Map file not found: {}", map_path);
                eprintln!("Checked both '{}' and '{}'", map_path, alt_path_str);
                std::process::exit(1);
            }
        }
    }

    if let Some(diff_str) = args
        .difficulty
        .as_ref()
        .filter(|s| Difficulty::from_str(s).is_err())
    {
        eprintln!("ERROR: Invalid difficulty: {}", diff_str);
        let valid: Vec<String> = Difficulty::all().map(|d| d.to_string()).collect();
        eprintln!("Valid difficulties: \n  {}", valid.join("\n  "));
        std::process::exit(1);
    }
    // ------------------

    unhaunter::wasm::app_run(CliOptions {
        include_draft_maps: args.draft_maps,
        net_mode,
        map_path: final_map_path,
        difficulty_id: args.difficulty,
        verbose: args.verbose,
        mute: args.mute,
    });
}
