use crate::qemu_args::{Arch, GuestOS};
use std::ffi::OsString;

impl Arch {
    pub fn matches_host(&self) -> bool {
        match self {
            Self::x86_64 => cfg!(target_arch = "x86_64"),
            Self::aarch64 => cfg!(target_arch = "aarch64"),
            Self::riscv64 => cfg!(target_arch = "riscv64"),
        }
    }

    pub fn machine_type(&self, guest_os: &GuestOS) -> OsString {
        match self {
            Self::x86_64 => {
                let machine_type = match guest_os {
                    GuestOS::Windows | GuestOS::WindowsServer | GuestOS::MacOS { .. } => "q35,hpet=off",
                    GuestOS::FreeDOS | GuestOS::Batocera | GuestOS::Haiku | GuestOS::Solaris | GuestOS::ReactOS | GuestOS::KolibriOS => "pc",
                    _ => "q35",
                };

                #[cfg(not(target_os = "macos"))]
                let smm = match guest_os {
                    GuestOS::Windows | GuestOS::WindowsServer | GuestOS::FreeDOS => "on",
                    _ => "off",
                };

                #[cfg(target_os = "macos")]
                let smm = "off";

                let mut machine = OsString::from(machine_type);
                machine.push(",smm=");
                machine.push(smm);
                machine.push(",vmport=off");
                machine
            }
            Self::aarch64 => "virt,virtualization=on,pflash0=rom,pflash1=efivars".into(),
            Self::riscv64 => "virt,usb=on".into(),
        }
    }
}
