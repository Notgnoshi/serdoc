mod config;

use clap::Parser;
use figment::providers::{Format, Serialized, Toml};
use figment::Figment;

use crate::config::Config;

fn main() {
    let cli_config = config::CliConfig::parse();
    println!("CLI config: {cli_config:?}");

    let layer1 = cli_config.config_dir.join("layer1.toml");
    let layer2 = cli_config.config_dir.join("layer2.toml");
    let overlaid_config: Config = Figment::new()
        .merge(Toml::file(layer1))
        .merge(Toml::file(layer2))
        // TODO: This doesn't work to override the TOML files from the CLI. See: https://github.com/SergioBenitez/Figment/issues/81
        .join(Serialized::defaults(cli_config.config))
        .extract()
        .unwrap();

    println!("Overlaid config: {overlaid_config:?}");
}
