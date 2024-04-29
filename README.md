## Quickemu - rewritten in Rust

This project is a rewrite of the [quickemu](https://github.com/quickemu-project/quickemu) bash script in Rust.
Backwards compatibility will be mostly maintained, but this project will diverge and may have different requirements.

This project depends on QEMU version 7.0.0 or later. Supporting older releases would require quite a bit of extra work,
and few people still use these releases. If your system has an older version of QEMU, this project is not for you. 

Currently, the requirements for QEMU features are the same as the original quickemu project. This may change, however.
It is recommended to use a build of QEMU that enables all of the features that would be used in GUI frontends to QEMU.
The requirement of certain QEMU features will not be treated as a bug.


## Planned features

1. **Support for multiple architectures**: The original quickemu script only supports the x86_64 architecture, both for 
the host and guest. This project aims to support multiple architectures, starting with x86_64, aarch64, and riscv64.
2. **Integration with libvirt**: This project primarily focuses on directly passing arguments to QEMU, but in the future,
it should be able to create XML files for use within libvirt. This will allow quickemu VMs to be managed through software
such as `virt-manager` or `gnome-boxes`. 
3. **GPU Passthrough**: A high priority of this project is to support passthrough of PCI devices. GPU passthrough
should be entirely handled for the user. 
