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
    //
    // However, don't Use this as the bottom layer in the stack until the end, so that we can
    // detect missing required fields.
    let defaults = Figment::new().merge(Serialized::defaults(Config::default()));
    let mut builder = Figment::new();

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
    builder = builder.merge(Serialized::defaults(cli.nullable_config));

    // Check for missing required values before adding the default values as the bottom layer.
    for field in Config::REQUIRED_FIELDS {
        if builder.find_value(field).is_err() {
            anyhow::bail!("Missing required config value '{field}'");
        }
    }

    // Insert the default values as the bottom layer
    builder = defaults.merge(builder);

    let overlaid_config: Config = builder.extract()?;

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
        let mut expected = Config::default();

        fixture.cli.nullable_config.required1 = Some(String::from(""));
        let actual = get_layered_configs_from_cli(fixture.cli.clone()).unwrap();
        assert_eq!(actual, expected);

        writeln!(fixture.layer1, "placeholder1 = 50").unwrap();
        expected.placeholder1 = 50;
        let actual = get_layered_configs_from_cli(fixture.cli.clone()).unwrap();
        assert_eq!(actual, expected);

        fixture.cli.no_config = true;
        expected = Config::default();
        let actual = get_layered_configs_from_cli(fixture.cli.clone()).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn required_fields_must_be_set() {
        let mut fixture = ConfigFixture::new().unwrap();

        // required1 not set, should return an Err
        let result = get_layered_configs_from_cli(fixture.cli.clone());
        assert!(result.is_err());

        // Set from a config file, even if to an empty string
        writeln!(fixture.layer1, "required1 = \"\"").unwrap();
        let result = get_layered_configs_from_cli(fixture.cli.clone());
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = Config::default();
        assert_eq!(actual, expected);

        // Set from the CLI
        let mut fixture = ConfigFixture::new().unwrap();
        fixture.cli.nullable_config.required1 = Some(String::from("foo"));
        let result = get_layered_configs_from_cli(fixture.cli.clone());
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = Config {
            required1: String::from("foo"),
            ..Default::default()
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn each_layer_overrides_the_previous() {
        let mut fixture = ConfigFixture::new().unwrap();
        let mut expected = Config::default();

        writeln!(fixture.layer1, "required1 = \"\"").unwrap();
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
        expected.required1 = String::from("bar");
        let actual = get_layered_configs_from_cli(fixture.cli.clone()).unwrap();
        assert_eq!(actual, expected);
    }
}
