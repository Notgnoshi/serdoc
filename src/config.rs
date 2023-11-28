use std::path::PathBuf;

use clap::Parser;
use documented::{Documented, DocumentedFields};
use serde::{Deserialize, Serialize};
use struct_field_names_as_array::FieldNamesAsSlice;

#[derive(Debug, Parser)]
pub struct CliConfig {
    /// Path to config directory
    #[clap(short = 'C', long, default_value = "./configs/")]
    pub config_dir: PathBuf,

    /// Disable reading config files from the --config-dir
    #[clap(long)]
    pub no_config: bool,

    // The rest of the CLI arguments
    #[clap(flatten)]
    pub nullable_config: NullableConfig,
}

/// Application configuration
#[derive(Debug, Clone, Documented, DocumentedFields, Deserialize, Serialize, FieldNamesAsSlice)]
pub struct Config {
    /// A placeholder config option
    pub placeholder1: u32,

    /// Another placeholder config option
    pub placeholder2: String,
}

// NOTE: This is the only place you should add default values for Config or NullableConfig fields.
impl Default for Config {
    fn default() -> Config {
        Config {
            placeholder1: 42,
            placeholder2: String::from("example"),
        }
    }
}

/// Application configuration
#[derive(Debug, Clone, Documented, DocumentedFields, Serialize, FieldNamesAsSlice, Parser)]
pub struct NullableConfig {
    /// A placeholder config option
    #[clap(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder1: Option<u32>,

    /// Another placeholder config option
    #[clap(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder2: Option<String>,
}

impl Config {
    #[cfg(test)]
    fn docstring_to_toml_comment(docstring: &str) -> String {
        let mut toml_comment = String::new();
        for line in docstring.lines() {
            let toml_line = if line.is_empty() {
                String::from("#\n")
            } else {
                format!("# {line}\n")
            };
            toml_comment.push_str(&toml_line);
        }
        toml_comment
    }

    /// Serialize a `Config` to a TOML document, where each field is prefixed with its
    /// documentation
    #[cfg(test)]
    fn to_documented_toml(&self) -> anyhow::Result<String> {
        let mut doc = toml_edit::ser::to_document(self)?;

        // Add the field docstring to the TOML document as a key prefix
        for (idx, (mut key, _value)) in doc.iter_mut().enumerate() {
            let docstring = Config::get_field_comment(key.get())?;
            let mut toml_comment = Self::docstring_to_toml_comment(docstring);

            // Add the docstring for the struct itself as a prefix to the very first key in the
            // document.
            if idx == 0 {
                let overview = Self::docstring_to_toml_comment(Self::DOCS);
                toml_comment = format!("{overview}\n{toml_comment}");
            }
            // Otherwise add a new line to ensure each field is visually separated
            else {
                toml_comment = format!("\n{toml_comment}");
            }
            let decor = key.decor_mut();
            decor.set_prefix(toml_comment);
        }
        let toml_str = doc.to_string();
        Ok(toml_str)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use super::*;

    #[test]
    fn every_config_field_has_documentation() {
        assert!(!Config::DOCS.is_empty(), "Config missing documentation");

        for field in Config::FIELD_NAMES_AS_SLICE {
            let docstring = Config::get_field_comment(field);
            assert!(docstring.is_ok(), "Config::{field} missing documentation");
            assert!(
                !docstring.unwrap().is_empty(),
                "Config::{field} has empty docstring"
            );
        }
    }

    #[test]
    fn nullable_config_matches_config() {
        assert_eq!(
            Config::FIELD_NAMES_AS_SLICE,
            NullableConfig::FIELD_NAMES_AS_SLICE,
            "Config and NullableConfig must have the same fields, in the same order"
        );
        assert_eq!(
            Config::DOCS,
            NullableConfig::DOCS,
            "Config and NullableConfig's documentation must be identical"
        );
        assert_eq!(
            Config::FIELD_DOCS,
            NullableConfig::FIELD_DOCS,
            "Config and NullableConfig field's must have the same documentation"
        );
    }

    #[test]
    fn default_docs_file_matches_default_impl() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = format!("{manifest_dir}/configs/default.toml");
        let mut file = std::fs::File::open(path).unwrap();
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents).unwrap();

        let default_config = Config::default();
        let expected_contents = default_config.to_documented_toml().unwrap();

        pretty_assertions::assert_eq!(
            expected_contents,
            file_contents,
            "\nIf this assertion fauls, run\n\
             \tcargo test -- --ignored generate_default_docs_file\n\
             to regenerate the default configuration documentation file\n",
        );
    }

    #[test]
    #[ignore = "Run this 'test' to generate the default.toml file"]
    fn generate_default_docs_file() {
        let default_config = Config::default();
        let default_contents = default_config.to_documented_toml().unwrap();
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = format!("{manifest_dir}/configs/default.toml");
        let mut file = std::fs::File::create(path).unwrap();
        file.write_all(default_contents.as_bytes()).unwrap();
    }
}
