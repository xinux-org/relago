use unic_langid::{LanguageIdentifier, langid};

include!(concat!(env!("OUT_DIR"), "/static_cache.rs"));

pub static DEFAULT_LC: LanguageIdentifier = langid!("en-US");

pub fn detect_language() -> LanguageIdentifier {
    std::env::var("LC_LANG")
        .or_else(|_| std::env::var("LANG"))
        .map_err(anyhow::Error::from)
        .map(|lang| {
            lang.split_once('.')
                .map(|(lang, _)| lang.to_string())
                .unwrap_or(lang)
        })
        .and_then(|lang| {
            lang.parse::<LanguageIdentifier>()
                .map_err(anyhow::Error::from)
        })
        .unwrap_or_else(|_| DEFAULT_LC.clone())
}
