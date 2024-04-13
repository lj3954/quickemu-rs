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
