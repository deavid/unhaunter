use clap::Parser;
use std::str::FromStr;
use undifficulty_core::difficulty::Difficulty;
use unhaunter::app_args::AppArgs;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, action)]
    draft_maps: bool,

    #[clap(long)]
    peer_host: Option<u16>,


    #[clap(long)]
    join: Option<String>,

    #[clap(long)]
    map: Option<String>,

    #[clap(long)]
    difficulty: Option<String>,

    #[clap(long)]
    installation_id_file: Option<String>,

    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[clap(long, action)]
    mute: bool,

    #[clap(long)]
    hub_url: Option<String>,

    #[clap(long)]
    cert: Option<String>,

    #[clap(long)]
    key: Option<String>,

    #[clap(long, action)]
    skip_ssl_verification: bool,
}

fn main() {
    let args = Args::parse();

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Err(e) = unassetidx_updater::update_assetidx_files() {
            eprintln!("Failed to update assetidx files: {}", e);
        }
    }

    let net_mode = if let Some(port) = args.peer_host {
        unhaunter::app_args::CliNetMode::PeerHost {
            port,
        }
    } else if let Some(address) = args.join {
        unhaunter::app_args::CliNetMode::Join {
            address,
            ticket: None,
        }
    } else {
        unhaunter::app_args::CliNetMode::Offline
    };

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

    unhaunter::wasm::app_run(AppArgs {
        verbose: args.verbose,
        mute: args.mute,
        include_draft_maps: args.draft_maps,
        net_mode,
        installation_id_file: args.installation_id_file,
        dedicated: false,
        procman_channel: None,
        hub_url: args.hub_url,
        cert_file: args.cert,
        key_file: args.key,
        skip_ssl_verification: args.skip_ssl_verification,
    });
}
