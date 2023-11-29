use clap::Parser;
use figment::providers::{Format, Serialized, Toml};
use figment::Figment;

use crate::config::{CliConfig, Config};

pub fn get_layered_configs() -> anyhow::Result<Config> {
    let cli_config = CliConfig::try_parse()?;
    let config = get_layered_configs_from_cli(cli_config)?;
    Ok(config)
}

pub fn get_layered_configs_from_cli(cli: CliConfig) -> anyhow::Result<Config> {
    // Use Config::default() as the base layer, so that at every layer, we can successfully
    // serialize to a Config, as well as being able to leave the config files and CLI args empty.
    let mut builder = Figment::new().merge(Serialized::defaults(Config::default()));

    // Only require the config dir to exist if we're using it
    if !cli.no_config {
        if !cli.config_dir.exists() {
            anyhow::bail!(
                "Config directory: {} does not exist",
                cli.config_dir.display()
            );
        }
        // This doesn't require the config files to exist, or for them to contain anything.
        let layer1 = cli.config_dir.join("layer1.toml");
        let layer2 = cli.config_dir.join("layer2.toml");
        builder = builder.merge(Toml::file(layer1)).merge(Toml::file(layer2));
    }
    // Finally, overlay the nullable CLI arguments over the top. The CLI config is nullable, so
    // that only CLI arguments that are provided by the user are merged with the bottom layers.
    let overlaid_config: Config = builder
        .merge(Serialized::defaults(cli.nullable_config))
        .extract()?;

    // TODO: Is there a way for required fields to be non-Options in the final deserialized struct?
    if overlaid_config.required1.is_none() {
        anyhow::bail!("Missing required config value 'required1'");
    }

    Ok(overlaid_config)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use tempdir::TempDir;

    use super::*;
    use crate::config::NullableConfig;

    struct ConfigFixture {
        dir: TempDir,
        layer1: File,
        layer2: File,
        cli: CliConfig,
    }

    impl ConfigFixture {
        fn new() -> anyhow::Result<ConfigFixture> {
            let dir = TempDir::new("configs")?;
            let layer1 = dir.path().join("layer1.toml");
            let layer1 = File::create(layer1)?;
            let layer2 = dir.path().join("layer2.toml");
            let layer2 = File::create(layer2)?;
            let cli = CliConfig {
                config_dir: dir.path().to_path_buf(),
                no_config: false,
                nullable_config: NullableConfig::default(),
            };

            Ok(ConfigFixture {
                dir,
                layer1,
                layer2,
                cli,
            })
        }
    }

    #[test]
    fn config_dir_doesnt_exist() {
        let fixture = ConfigFixture::new().unwrap();
        drop(fixture.dir);
        let result = get_layered_configs_from_cli(fixture.cli);
        assert!(result.is_err());
    }

    #[test]
    fn disable_config_files() {
        let mut fixture = ConfigFixture::new().unwrap();
        let mut expected = Config {
            required1: Some("foo".into()),
            ..Default::default()
        };

        fixture.cli.nullable_config.required1 = Some("foo".into());
        let actual = get_layered_configs_from_cli(fixture.cli.clone()).unwrap();
        assert_eq!(actual, expected);

        writeln!(fixture.layer1, "placeholder1 = 50").unwrap();
        expected.placeholder1 = 50;
        let actual = get_layered_configs_from_cli(fixture.cli.clone()).unwrap();
        assert_eq!(actual, expected);

        fixture.cli.no_config = true;
        expected = Config::default();
        expected.required1 = Some("foo".into());
        let actual = get_layered_configs_from_cli(fixture.cli.clone()).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn each_layer_overrides_the_previous() {
        let mut fixture = ConfigFixture::new().unwrap();
        let mut expected = Config {
            required1: Some("foo".into()),
            ..Default::default()
        };
        writeln!(fixture.layer1, "required1 = \"foo\"").unwrap();

        let actual = get_layered_configs_from_cli(fixture.cli.clone()).unwrap();
        assert_eq!(actual, expected);

        writeln!(fixture.layer1, "placeholder1 = 50").unwrap();
        expected.placeholder1 = 50;
        let actual = get_layered_configs_from_cli(fixture.cli.clone()).unwrap();
        assert_eq!(actual, expected);

        writeln!(fixture.layer2, "placeholder1 = 100").unwrap();
        expected.placeholder1 = 100;
        let actual = get_layered_configs_from_cli(fixture.cli.clone()).unwrap();
        assert_eq!(actual, expected);

        fixture.cli.nullable_config.placeholder1 = Some(1000);
        expected.placeholder1 = 1000;
        let actual = get_layered_configs_from_cli(fixture.cli.clone()).unwrap();
        assert_eq!(actual, expected);

        // IMPORTANT: 42 is the default value. Even then, we expect it to overwrite whatever is in
        // the config files! (This is tricky, and is why NullableConfig exists)
        fixture.cli.nullable_config.placeholder1 = Some(42);
        expected.placeholder1 = 42;
        let actual = get_layered_configs_from_cli(fixture.cli.clone()).unwrap();
        assert_eq!(actual, expected);

        fixture.cli.nullable_config.required1 = Some("bar".into());
        expected.required1 = Some("bar".into());
        let actual = get_layered_configs_from_cli(fixture.cli.clone()).unwrap();
        assert_eq!(actual, expected);
    }
}
