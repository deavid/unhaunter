use bevy_replicon::prelude::ProtocolHash;
use clap::Parser;
use unhaunter::app_args::AppArgs;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, action)]
    draft_maps: bool,

    #[clap(long, default_value_t = 5000)]
    host: u16,

    #[clap(long)]
    procman_channel: Option<String>,

    #[clap(long)]
    hub_url: Option<String>,

    #[clap(long)]
    bind: Vec<String>,

    #[clap(long)]
    installation_id_file: Option<String>,

    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[clap(long, action)]
    print_version: bool,

    #[clap(long, action)]
    print_protocol_hash: bool,

    #[clap(long)]
    cert: Option<String>,

    #[clap(long)]
    key: Option<String>,

    #[clap(long, action)]
    skip_ssl_verification: bool,
}

fn main() {
    let args = Args::parse();

    let bind_addresses = args.bind.clone();

    let net_mode = unhaunter::app_args::CliNetMode::PeerHost {
        port: args.host,
        bind_addresses,
    };

    if args.print_version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if args.print_protocol_hash {
        let mut app = unhaunter::app::app_build(AppArgs {
            verbose: args.verbose,
            mute: true,
            include_draft_maps: args.draft_maps,
            net_mode,
            installation_id_file: args.installation_id_file,
            dedicated: true,
            procman_channel: args.procman_channel.clone(),
            hub_url: args.hub_url.clone(),
            cert_file: args.cert.clone(),
            key_file: args.key.clone(),
            skip_ssl_verification: args.skip_ssl_verification,
        });

        app.finish();
        app.cleanup();

        let protocol_hash = serde_json::to_string(app.world().resource::<ProtocolHash>())
            .expect("Error serializing protocol hash");

        println!("{}", protocol_hash);
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Err(e) = unassetidx_updater::update_assetidx_files() {
            eprintln!("Failed to update assetidx files: {}", e);
        }
    }

    println!(
        "Starting Unhaunter Dedicated Server on port {}...",
        args.host
    );

    unhaunter::app::app_run(AppArgs {
        verbose: args.verbose,
        mute: true,
        include_draft_maps: args.draft_maps,
        net_mode,
        installation_id_file: args.installation_id_file,
        dedicated: true,
        procman_channel: args.procman_channel,
        hub_url: args.hub_url,
        cert_file: args.cert,
        key_file: args.key,
        skip_ssl_verification: args.skip_ssl_verification,
    });
}
