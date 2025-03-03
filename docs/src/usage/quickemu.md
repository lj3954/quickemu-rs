# Quickemu

Quickemu is responsible for launching virtual machines using configuration files.

Configuration files are TOML formatted, information about their usage can be found in
the [configuration docs](../configuration/configuration.md).

## Usage

Run `quickemu-rs` followed by a path to a configuration file.

For example,
```bash
quickemu-rs ubuntu-24.04-x86_64.toml
```
