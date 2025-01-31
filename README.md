## Quickemu-rs

Create and manage macOS, Linux, and Windows virtual machines

This project depends on QEMU version 7.0.0 or later. Supporting older releases would require quite a bit of extra work,
and few people still use these releases. If your system has an older version of QEMU, this project is not for you. 

The feature flag `qemu_8_1` requires QEMU 8.1.0 or later, and is enabled by default. If you have a previous version of QEMU,
building from source will be required until a binary is provided without the feature flag.

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

Usage information is contained within the [**project's wiki**](https://github.com/lj3954/quickemu-rs/wiki). Please refer to it for more information, including how to configure quickemu.

## Licensing

All parts of quickget-rs are licensed under the GPLv3-only license. Full text can be found in the LICENSE-GPLv3 file.
Quickemu-rs is dual licensed under GPLv2-only and GPLv3-only. This is done to allow QEMU to be statically linked with the produced binary in the future, simplifying distribution of quickemu-rs in containerized formats or where a builtin QEMU is otherwise wanted.

## Planned features

1. **Integration with libvirt**: This project primarily focuses on directly passing arguments to QEMU, but in the future,
it should be able to create XML files for use within libvirt. This will allow quickemu VMs to be managed through software
such as `virt-manager` or `gnome-boxes`. 
2. **GPU Passthrough**: A high priority of this project is to support passthrough of PCI devices. GPU passthrough
should be entirely handled for the user. 
