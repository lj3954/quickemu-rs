## Quickemu - rewritten in Rust

This project is a rewrite of the [quickemu](https://github.com/quickemu-project/quickemu) bash script in Rust.
Backwards compatibility will be mostly maintained, but this project will diverge and may have different requirements.

This project depends on QEMU version 7.0.0 or later. Supporting older releases would require quite a bit of extra work,
and few people still use these releases. If your system has an older version of QEMU, this project is not for you. 

Currently, the requirements for QEMU features are the same as the original quickemu project. This may change, however.
It is recommended to use a build of QEMU that enables all of the features that would be used in GUI frontends to QEMU.
The requirement of certain QEMU features will not be treated as a bug.

## Installation

This project is in early stages, so binaries are not available. To install, you must build the package from this source.
The only build dependency is rust. Many distros package the language, though the recommended way to install rust is through
the [rustup](https://www.rust-lang.org/tools/install) script. After installing rust, you can build the project using cargo.

```bash
git clone https://github.com/lj3954/quickemu-rs.git
cd quickemu-rs
cargo build --release
```

The binary will be located in the `target/release` directory. You can move this binary to your desired location.

## Usage

TODO. Use the documentation provided by quickemu for now, most features should be nearly the same.


## Planned features

1. **Integration with libvirt**: This project primarily focuses on directly passing arguments to QEMU, but in the future,
it should be able to create XML files for use within libvirt. This will allow quickemu VMs to be managed through software
such as `virt-manager` or `gnome-boxes`. 
2. **GPU Passthrough**: A high priority of this project is to support passthrough of PCI devices. GPU passthrough
should be entirely handled for the user. 
