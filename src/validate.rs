use anyhow::Result;

// QEMU supported keyboard layouts. According to the documentation, these are unnecessary
// (excluding macOS hosts, where they will always be used)
// https://www.qemu.org/docs/master/system/qemu-manpage.html
const KEYBOARD_LAYOUTS: [&str; 33] = ["ar", "de-ch", "es", "fo", "fr-ca", "hu", "ja", "mk", "no", "pt-br", "sv",
                                    "da", "en-gb", "et", "fr", "fr-ch", "is", "lt", "nl", "pl", "ru", "th",
                                    "de", "en-us", "fi", "fr-be", "hr", "it", "lv", "nl-be", "pt", "sl", "tr"];

pub fn validate_keyboard_layout(layout: String) -> Result<String> {
    if KEYBOARD_LAYOUTS.contains(&layout.as_str()) {
        Ok(layout)
    } else {
        Err(anyhow::anyhow!("Keyboard layout {} is not supported by QEMU.", layout))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn access() {
        use crate::config::Access;
        let remote = Access::from(Some("remote".into()));
        assert_eq!(remote, Access::Remote);
        let local = Access::from(Some("local".into()));
        assert_eq!(local, Access::Local);
        let address = Access::from(Some("1.2.3.4".into()));
        assert_eq!(address, Access::Address("1.2.3.4".to_string()));
        let no_input = Access::from(None);
        assert_eq!(no_input, Access::Local);
    }

    #[test]
    fn cpu_cores() {
        use crate::config_parse::cpu_cores;
        let incl_input_no_ht = cpu_cores(Some("4".into()), 8, 16);
        assert_eq!(incl_input_no_ht.unwrap(), (4, false));
        let normal_input = cpu_cores(Some("8".into()), 16, 8);
        assert_eq!(normal_input.unwrap(), (8, true));
        let nine_cores = cpu_cores(None, 9, 4);
        assert_eq!(nine_cores.unwrap(), (4, true));
        let invalid_cores = cpu_cores(None, 8, 16);
        assert!(invalid_cores.is_err());
        let one_core = cpu_cores(None, 1, 1);
        assert_eq!(one_core.unwrap(), (1, false));
    }
}
