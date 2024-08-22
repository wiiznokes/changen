use changelog_gen::{config::Cli, run};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    run(cli)
}
