# Installation

## Arch Linux

quickemu-rs is officially maintained on the AUR; you can install it with your favourite AUR helper.

For instance,
```bash
paru -S quickemu-rs
```

## Dependencies

quickemu-rs depends on QEMU, which must be compiled with GTK, SDL, SPICE, and VirtFS support.
By default, smartcard support is also required, and the minimum QEMU version is 8.1.0.

If you are [building from source](./installation.md#compilation-from-source), you can optionally disable the `smartcard_args` and `qemu_8_1` features,
which will remove these requirements.

## Binaries

Pre-built binaries are available for macOS and GNU/Linux on the [releases page][releases].
To install, download the archive for your platform, extract it, and copy the binaries to a directory in your `$PATH`,
such as `~/.local/bin`.

## Compilation from source

Alternatively, you can manually compile from source. To do so, you will need to have Rust installed.
For more information on how to install Rust, see the [official Rust website][rust-install].

Once you have installed Rust, you can clone the repository and build the project:
```bash
git clone https://github.com/lj3954/quickemu-rs.git
cd quickemu-rs
cargo build --release
```

This will compile the 2 binaries into the `target/release` directory. Then, you can copy the binaries into a directory in your `$PATH`.

[releases]: https://github.com/lj3954/quickemu-rs/releases
[rust-install]: https://www.rust-lang.org/tools/install
