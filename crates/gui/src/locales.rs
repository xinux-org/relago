use unic_langid::LanguageIdentifier;

include!(concat!(env!("OUT_DIR"), "/static_cache.rs"));

pub fn try_detect_language() -> Option<LanguageIdentifier> {
    let mut lang = std::env::var("LC_LANG");

    if lang.is_err() {
        lang = std::env::var("LANG");
    }

    if let Ok(lang) = lang {
        return lang.parse::<LanguageIdentifier>().ok();
    }

    None
}
