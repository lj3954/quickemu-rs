use std::sync::LazyLock;

use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DefaultLocalizer, DesktopLanguageRequester, LanguageLoader, Localizer,
};
use rust_embed::RustEmbed;

fn init(loader: &FluentLanguageLoader) {
    let requested_languages = DesktopLanguageRequester::requested_languages();

    let localizer = DefaultLocalizer::new(loader, &Localizations);
    if let Err(e) = localizer.select(&requested_languages) {
        log::warn!("Failed to load localizations: {e}");
    }
}

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader = fluent_language_loader!();
    loader
        .load_fallback_language(&Localizations)
        .expect("Error while loading fallback language");
    init(&loader);

    loader
});

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

#[macro_export]
macro_rules! fl_bail {
    ($message_id:literal) => {{
        anyhow::bail!(i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id))
    }};

    ($message_id:literal, $($args:expr),*) => {{
        anyhow::bail!(i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id, $($args), *))
    }};
}

#[macro_export]
macro_rules! fl_ensure {
    ($condition:expr, $message_id:literal) => {{
        anyhow::ensure!(
            $condition,
            i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id)
        )
    }};
    ($condition:expr, $message_id:literal, $($args:expr),*) => {{
        anyhow::ensure!(
            $condition,
            i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id, $($args), *)
        )
    }};
}
