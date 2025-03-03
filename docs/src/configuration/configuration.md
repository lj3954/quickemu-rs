Quickemu can be configured primarily by editing the configuration file, which is TOML formatted.

# Guest OS

Specifying a Guest OS allows quickemu to select compatible hardware devices, optimize performance, and
enable features supported on different guest operating systems.

The field can be edited as so.

```toml
[guest]
os = "macos"
# On certain operating systems, other fields can be present here.
# For example, macOS requires a release to select compatible emulated hardware.
release = "sequoia"
```

# VM Directory and Name

The VM's name can be set through the 'vm_name' entry in the configuration file.

The VM's directory can be set through the 'vm_dir' entry.

If either are unpopulated, they will be set based on the other, or if unavailable,
the configuration file's name.

# Machine configuration

Quickemu allows for various customizations to the emulated machine.

All options within this category must be under [machine] in your configuration file.

## CPU threads

Quickemu will automatically detect whether your CPU supports SMT, and
configure your VM in the same way. On a system with SMT, an odd number of threads
will be rounded down.

You can set the amount of total CPU threads the VM should receive as such.

```toml
cpu_threads = 8
```

## RAM

Quickemu supports configuring sizes, such as RAM, using both integers (in bytes), as well as
strings representing a size, with binary-prefix units. 

For example, the configuration below will allocate 8 GiB of memory to your VM.

```toml
ram = "8G"
```

## Boot Types

Quickemu defaults to EFI boot with secure boot disabled.
Legacy BIOS and Secure Boot are both supported.

You can configure quickemu to use legacy BIOS as follows

```toml
[machine.boot]
type = "legacy"
```

Alternatively, to enable secure boot, you can modify the configuration as follows.
Note that you must explictly specify the EFI boot type in order to enable secure boot.

```toml
[machine.boot]
type = "efi"
secure_boot = true
```

## TPM

Quickemu supports TPM 2.0, using swtpm for emulation. It can be configured as follows

```toml
tpm = true
```

## Status Quo

This option marks all disks attached to the VM as read-only,
ensuring they are not modified while the VM is running.

```toml
status_quo = true
```

# IO devices

Quickemu supports configuration of the IO devices emulated to the guest.

All options within this category must be under [io] in your configuration file.

## Overriding Guest-selected IO devices

### USB Controller

Certain guests (in specific, macOS) may break with non-default USB controllers.
You can also disable the USB controller:

```toml
usb_controller = "none"
```

### Keyboard

```toml
keyboard = "virtio"
```

### Mouse

```toml
mouse = "tablet"
```

### Sound Card

```toml
sound_card = "intel_hda"
```

## Keyboard Layout

Quickemu, through QEMU, supports emulating various keyboard layouts to the guest.
The default is 'en-us'

```toml
keyboard_layout = "fr"
```

## Public Directory

You can modify which directory is shared with the VM. The default currently is
~/Public.
You can also disable sharing a directory:

```toml
public_dir = "none"
```

# Display

Display is a subcategory of io. Therefore, all display configuration must be put under [io.display]

## Display Types

Available display types are as follows

none, sdl, gtk, spice, spice_app, cocoa

Cocoa is specific to macOS, while quickemu builds without spice support on
macOS targets due to the lack of spice support in the homebrew QEMU package.

### Spice

Spice displays (not to be confused with spice app) have certain other configuration options.

```toml
type = "spice"
# The port spice can be accessed through
spice_port = 5930
# The IP address spice can be accessed through
access = "127.0.0.1"
# The spice viewer for quickemu to launch after the VM starts
viewer = "remote"
```

## Resolution

Resolution can be set in multiple ways.

You can fully customize it with a width & height as follows:

```toml
[io.display.resolution]
type = "custom"
width = 1920
height = 1080
```

You can also enable fullscreen

```toml
[io.display.resolution]
type = "fullscreen"
```

Or (with the display_resolution feature flag - enabled by default):

```toml
[io.display.resolution]
type = "display"
# Optionally, set which display to base the resolution off
# display_name = "Example"
# And, a percentage of the display
# percentage = 60.5
```

## Acceleration

Hardware acceleration in the guest can be manually enabled as follows:

```toml
accelerated = true
```

Note that some guest OSes (e.g. macOS) may override this option due to their reliance upon
emulated display devices which do not support hardware acceleration.

## Braille

Quickemu can output braille when this option is set

```toml
braille = true
```

# Images

All paths can be either absolute or relative to the VM directory

## ISO/IMG

ISO and IMG files can be mounted as follows:

```toml
[[images.iso]]
path = "ubuntu-24.04.iso"

[[images.img]]
path = "RecoveryImage.img"
```

Alongside the path, you can add 'always_mount = true' if the image is to be mounted
even after the operating system is installed (determined through disk size).

## Disks

Disk images can be mounted as follows:

```toml
path = "disk.qcow2"
# Optional; default dependent on Guest OS
# Size, much like RAM (read above) can be either set as an integer number of bytes
# or a string representing a size with binary-prefix units
size = "30G"
# Optional; defaults to qcow2
format = "raw"
# Optional; defaults to off. NOTE: Requires disk format to be set
preallocation = "full"
```

Disks will be created using qemu-img if they do not already exist.

# Networking

All options here must be placed under [network] in your config file.

## Disable networking

```toml
type = "none"
```

## NAT

```toml
type = "nat"
# Set a desired SSH port. By default, 22220 will be used
ssh_port = 22220
# Restrict networking to only the guest and virtual devices
restrict = true

# Set each port forward in the array like this
[[network.port_forwards]]
host = 8080
guest = 8080
```

## Bridged

```toml
type = "bridged"
# You must specify a bridge interface
bridge = "br0"
# Optionally specify a mac address. Must be in the range 52:54:00:AB:00:00 - 52:54:00:AB:FF:FF
mac_addr = "52:54:00:AB:51:AE"
```

## Monitor and Serial

The QEMU monitor and serial outputs can each be manually configured.
By default, they will use unix sockets with a path in your VM directory.

### Socket

```toml
[network.serial]
type = "socket"
socketpath = "vm-socket.sock"
```

### Telnet

```toml
[network.monitor]
type = "telnet"
# The Telnet address has a default value unique to each monitor and serial.
# To manually specify, include a full socket address, including both an IP address and port
address = "127.0.0.1:4440"
```

