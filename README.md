# Serdoc

A simple application to experiment with layered config files and CLI overrides

## Requirements

* [X] Configuration directory is customizable
* [X] Config files can be disabled
* [X] `configs/default.toml` is generated from `Config::default()`
* [X] Fields in `default.toml` are documented with `Config`s docstrings
* [X] Multiple config files can be layered on top of each other, overriding settings found in the
      previous layer
* [X] CLI arguments can override any config file option
* [ ] Have config options that _must_ be set, either from a config file, or the CLI (don't fallback
      on the `Default` trait impl)
* [ ] Any layer overrides are logged
* [ ] Support passing the same CLI flag multiple times to form an array (or pass in a delimited
      value)
* [ ] Support more complex TOML file formats (nested tables)
