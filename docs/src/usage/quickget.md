# Quickget

Quickget is responsible for downloading operating system images from pre-generated URLs, and creating sensible
configurations which can be passed into quickemu.

> Since all available operating systems are fetched from the internet, quickget may be slow on its first launch
each day. Configurations are cached and will be refreshed if quickget hasn't been run since UTC midnight.

## Usage

Running `quickget-rs` without any arguments will list all available operating systems.
Then, you can pass in an operating system to list all available releases and (if applicable) editions.

Following that, you can easily download an operating system and create a configuration by passing in the
operating system, release, and edition (if applicable).

For example, to create a VM running Kubuntu 24.04 LTS:
```bash
quickget-rs kubuntu 24.04
```

### CLI arguments

| Argument | Description |
|----------|-------------|
| `-h`, `--help` | Print help message |
| `--verbose` | Enable verbose output |
| `-r`, `--refresh` | Force configuration data to be refreshed |
| `-a`, `--arch` | Specify the architecture of operating system you want to download. By default, quickget will select your system's architecture if possible |
| `-l`, `--list` | List all available operating systems, releases, and editions. By default, plain text will be printed. Passing `csv` or `json` will modify the formatting. This is mainly here for backwards compatibility |

<!-- TODO: Fix currently ignored arguments and add them to this manual -->
