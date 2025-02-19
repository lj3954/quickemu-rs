# Config errors
read-config-error = Could not read config file: { $err }
parse-config-error = Could not parse config file: { $err }

# Live VM errors
failed-live-vm-de = Failed to deserialize live VM data: { $err }
failed-del-live-file = Failed to delete inactive live VM status file: { $err }
failed-vm-kill = Failed to kill running VM: { $err }

# Monitor errors
no-monitor-available = No monitor is enabled.
failed-monitor-write = Could not write to the monitor: { $err }
failed-monitor-read = Could not read from thet monitor: { $err }

# Generic Errors
macos-cpu-instructions = CPU does not support a necessary instruction for this macOS release: { $instruction }.
unavailable-port = Requested port { $port } is not available.
insufficient-ram = System RAM { $ram } is insufficient for { $guest } VMs.
sound-usb-conflict = USB Audio requires the XHCI USB controller.
which-binary = Could not find binary: { $err }
failed-launch = Failed to launch { $bin }: { $err }
non-x86-bios = Legacy boot is only supported on x86_64.
riscv64-boot = Could not find riscv64 bootloader
efi-firmware = Could not find EFI firmware
failed-ovmf-copy = Could not copy OVMF vars into VM directory: { $err }
unsupported-boot-combination = Specified architecture and boot type are not compatible
no-viewer = Could not find viewer { $viewer_bin }
no-qemu = Could not find qemu binary { $qemu_bin }
failed-disk-creation = Could not create disk image: { $err }
disk-used = Failed to get write lock on disk { $disk }. Ensure that it is not already in use.
failed-qemu-img-deserialization = Could not deserialize qemu-img info: { $err }
no-mac-bootloader = Could not find macOS bootloader in VM directory
nonexistent-image = Requested to mount image { $img }, but it does not exist.
monitor-command-failed = Could not send command to monitor: { $err }
failed-live-vm-se = Failed to serialize live VM data: { $err }

# Warnings
macos-core-power-two = macOS guests may not boot witwh core counts that are not powers of two. Recommended rounding: { $recommended }.
software-virt-fallback = Hardware virtualization{ $virt_branding } is not enabled on your CPU. Falling back to software virtualization, performance will be degraded
audio-backend-unavailable = Sound was requested, but no audio backend could be detected.
insufficient-ram-configuration = The specified amount of RAM ({ $ram }) is insufficient for { $guest }. Performance issues may arise
