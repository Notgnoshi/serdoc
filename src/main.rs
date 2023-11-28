mod config;
use clap::Parser;

fn main() {
    let config = config::CliConfig::parse();

    println!("CLI config: {config:?}");
}
