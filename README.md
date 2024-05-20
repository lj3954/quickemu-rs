## Quickemu - rewritten in Rust

This project is a rewrite of the [quickemu](https://github.com/quickemu-project/quickemu) bash script in Rust.
Backwards compatibility will be mostly maintained, but this project will diverge and may have different requirements.

This project depends on QEMU version 7.0.0 or later. Supporting older releases would require quite a bit of extra work,
and few people still use these releases. If your system has an older version of QEMU, this project is not for you. 

Currently, the requirements for QEMU features are the same as the original quickemu project. This may change, however.
It is recommended to use a build of QEMU that enables all of the features that would be used in GUI frontends to QEMU.
The requirement of certain QEMU features will not be treated as a bug.

## Installation

Binary releases are available. Download one of the releases and extract it to a directory in your PATH, such as `~/.local/bin`. 

Alternatively, you can build the project from source. After installing rust, build it with cargo.


```bash
git clone https://github.com/lj3954/quickemu-rs.git
cd quickemu-rs
cargo build --release
```

Currently, on an aarch64 linux host, you will have to build the project from source. This is due to an issue with the build dependencies.

x86_64 linux and all macOS hosts should be able to use the binary releases. Please report an issue if a binary release
does not launch or if you're unable to launch a VM for any reason.

## Usage

TODO. Use the documentation provided by quickemu for now, most features should be nearly the same.


## Planned features

1. **Integration with libvirt**: This project primarily focuses on directly passing arguments to QEMU, but in the future,
it should be able to create XML files for use within libvirt. This will allow quickemu VMs to be managed through software
such as `virt-manager` or `gnome-boxes`. 
2. **GPU Passthrough**: A high priority of this project is to support passthrough of PCI devices. GPU passthrough
should be entirely handled for the user. 

## Known bugs

These will be fixed. Eventually. 

(UNIMPLEMENTED FEATURE) Spice display type is not yet supported.

(BUG) migrate-config puts disk size in bytes in TOML config, but my parser expects the format QEMU uses.
