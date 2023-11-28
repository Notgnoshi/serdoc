mod config;

use clap::Parser;
use figment::providers::{Format, Serialized, Toml};
use figment::Figment;

use crate::config::Config;

fn main() {
    let cli_config = config::CliConfig::parse();
    println!("CLI config: {cli_config:?}");

    let mut builder = Figment::new().merge(Serialized::defaults(Config::default()));
    if !cli_config.no_config {
        if !cli_config.config_dir.exists() {
            panic!(
                "Config directory: {} does not exist",
                cli_config.config_dir.display()
            );
        }
        let layer1 = cli_config.config_dir.join("layer1.toml");
        let layer2 = cli_config.config_dir.join("layer2.toml");
        builder = builder.merge(Toml::file(layer1)).merge(Toml::file(layer2))
    }
    let overlaid_config: Config = builder
        .merge(Serialized::defaults(cli_config.nullable_config))
        .extract()
        .unwrap();

    println!("Overlaid config: {overlaid_config:?}");
}
