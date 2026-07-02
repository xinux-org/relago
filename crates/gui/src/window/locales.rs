use unic_langid::{langid, LanguageIdentifier};

include!(concat!(env!("OUT_DIR"), "/static_cache.rs"));

pub const UZ: LanguageIdentifier = langid!("uz-UZ");
pub const RU: LanguageIdentifier = langid!("ru-RU");
pub const EN: LanguageIdentifier = langid!("en-US");
