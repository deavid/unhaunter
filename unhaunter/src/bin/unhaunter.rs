use clap::Parser;
use untypes_core::cli::CliOptions;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, action)]
    draft_maps: bool,

    #[clap(long)]
    host: Option<u16>,

    #[clap(long)]
    join: Option<String>,

    #[clap(long)]
    map: Option<String>,

    #[clap(long)]
    difficulty: Option<String>,
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
        untypes_core::cli::NetMode::Host { port }
    } else if let Some(address) = args.join {
        untypes_core::cli::NetMode::Join { address }
    } else {
        untypes_core::cli::NetMode::Offline
    };

    unhaunter::wasm::app_run(CliOptions {
        include_draft_maps: args.draft_maps,
        net_mode,
        map_path: args.map,
        difficulty_id: args.difficulty,
    });
}
