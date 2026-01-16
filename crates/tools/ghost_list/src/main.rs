use clap::Parser;
use ghost_list::cli::Cli;

fn main() {
    let cli = Cli::parse();
    cli.execute();
}
