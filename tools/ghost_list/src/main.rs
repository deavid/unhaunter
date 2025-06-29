use clap::Parser;
use ghost_list::Cli;

fn main() {
    let cli = Cli::parse();
    cli.execute();
}
