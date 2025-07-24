//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

///// Internationalization with Fluent and system language detection /////

use once_cell::unsync::OnceCell;
use fluent_bundle::{FluentBundle, FluentResource};
use std::thread_local;
use unic_langid::LanguageIdentifier;

thread_local! {
    pub static FLUENT_BUNDLE: OnceCell<FluentBundle<FluentResource>> = OnceCell::new();
}

#[cfg(feature = "lang_ar")]
pub static FTL_AR: &str = include_str!("../i18n/ar.ftl");
#[cfg(feature = "lang_da")]
pub static FTL_DA: &str = include_str!("../i18n/da.ftl");
#[cfg(feature = "lang_de")]
pub static FTL_DE: &str = include_str!("../i18n/de.ftl");
#[cfg(feature = "lang_el")]
pub static FTL_EL: &str = include_str!("../i18n/el.ftl");
#[cfg(feature = "lang_en")]
pub static FTL_EN: &str = include_str!("../i18n/en.ftl");
#[cfg(feature = "lang_es")]
pub static FTL_ES: &str = include_str!("../i18n/es.ftl");
#[cfg(feature = "lang_fi")]
pub static FTL_FI: &str = include_str!("../i18n/fi.ftl");
#[cfg(feature = "lang_fr")]
pub static FTL_FR: &str = include_str!("../i18n/fr.ftl");
#[cfg(feature = "lang_hi")]
pub static FTL_HI: &str = include_str!("../i18n/hi.ftl");
#[cfg(feature = "lang_it")]
pub static FTL_IT: &str = include_str!("../i18n/it.ftl");
#[cfg(feature = "lang_ja")]
pub static FTL_JA: &str = include_str!("../i18n/ja.ftl");
#[cfg(feature = "lang_ko")]
pub static FTL_KO: &str = include_str!("../i18n/ko.ftl");
#[cfg(feature = "lang_nl")]
pub static FTL_NL: &str = include_str!("../i18n/nl.ftl");
#[cfg(feature = "lang_no")]
pub static FTL_NO: &str = include_str!("../i18n/no.ftl");
#[cfg(feature = "lang_pl")]
pub static FTL_PL: &str = include_str!("../i18n/pl.ftl");
#[cfg(feature = "lang_pt")]
pub static FTL_PT: &str = include_str!("../i18n/pt.ftl");
#[cfg(feature = "lang_ru")]
pub static FTL_RU: &str = include_str!("../i18n/ru.ftl");
#[cfg(feature = "lang_sl")]
pub static FTL_SL: &str = include_str!("../i18n/sl.ftl");
#[cfg(feature = "lang_sv")]
pub static FTL_SV: &str = include_str!("../i18n/sv.ftl");
#[cfg(feature = "lang_sw")]
pub static FTL_SW: &str = include_str!("../i18n/sw.ftl");
#[cfg(feature = "lang_uk")]
pub static FTL_UK: &str = include_str!("../i18n/uk.ftl");
#[cfg(feature = "lang_zh")]
pub static FTL_ZH: &str = include_str!("../i18n/zh.ftl");

struct LangEntry {
    code: &'static str,
    #[allow(dead_code)]
    #[cfg(any(
        feature = "lang_ar", feature = "lang_da", feature = "lang_de", feature = "lang_el",
        feature = "lang_en", feature = "lang_es", feature = "lang_fi", feature = "lang_fr",
        feature = "lang_hi", feature = "lang_it", feature = "lang_ja", feature = "lang_ko",
        feature = "lang_nl", feature = "lang_no", feature = "lang_pl", feature = "lang_pt",
        feature = "lang_ru", feature = "lang_sl", feature = "lang_sv", feature = "lang_sw",
        feature = "lang_uk", feature = "lang_zh"
    ))]
    ftl: &'static str,
}

const LANGS: &[LangEntry] = &[
    #[cfg(feature = "lang_ar")]
    LangEntry { code: "ar", ftl: FTL_AR },
    #[cfg(feature = "lang_da")]
    LangEntry { code: "da", ftl: FTL_DA },
    #[cfg(feature = "lang_de")]
    LangEntry { code: "de", ftl: FTL_DE },
    #[cfg(feature = "lang_el")]
    LangEntry { code: "el", ftl: FTL_EL },
    #[cfg(feature = "lang_en")]
    LangEntry { code: "en", ftl: FTL_EN },
    #[cfg(feature = "lang_es")]
    LangEntry { code: "es", ftl: FTL_ES },
    #[cfg(feature = "lang_fi")]
    LangEntry { code: "fi", ftl: FTL_FI },
    #[cfg(feature = "lang_fr")]
    LangEntry { code: "fr", ftl: FTL_FR },
    #[cfg(feature = "lang_hi")]
    LangEntry { code: "hi", ftl: FTL_HI },
    #[cfg(feature = "lang_it")]
    LangEntry { code: "it", ftl: FTL_IT },
    #[cfg(feature = "lang_ja")]
    LangEntry { code: "ja", ftl: FTL_JA },
    #[cfg(feature = "lang_ko")]
    LangEntry { code: "ko", ftl: FTL_KO },
    #[cfg(feature = "lang_nl")]
    LangEntry { code: "nl", ftl: FTL_NL },
    #[cfg(feature = "lang_no")]
    LangEntry { code: "no", ftl: FTL_NO },
    #[cfg(feature = "lang_pl")]
    LangEntry { code: "pl", ftl: FTL_PL },
    #[cfg(feature = "lang_pt")]
    LangEntry { code: "pt", ftl: FTL_PT },
    #[cfg(feature = "lang_ru")]
    LangEntry { code: "ru", ftl: FTL_RU },
    #[cfg(feature = "lang_sl")]
    LangEntry { code: "sl", ftl: FTL_SL },
    #[cfg(feature = "lang_sv")]
    LangEntry { code: "sv", ftl: FTL_SV },
    #[cfg(feature = "lang_sw")]
    LangEntry { code: "sw", ftl: FTL_SW },
    #[cfg(feature = "lang_uk")]
    LangEntry { code: "uk", ftl: FTL_UK },
    #[cfg(feature = "lang_zh")]
    LangEntry { code: "zh", ftl: FTL_ZH },
];

pub fn get_system_language() -> String {
    fn extract_lang(s: &str) -> Option<String> {
        let first_part = s.split(&['_', '.']).next()?;
        if first_part.is_empty() {
            None
        } else {
            Some(first_part.to_string())
        }
    }

    for var in &["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = std::env::var(var) {
            if let Some(lang) = extract_lang(&val) {
                return lang;
            }
        }
    }

    "en".to_string()
}

pub fn load_fluent_bundle() -> Option<FluentBundle<FluentResource>> {
    let lang = get_system_language();

    let entry = LANGS
        .iter()
        .find(|e| e.code == lang)
        .or_else(|| LANGS.iter().find(|e| e.code == "en"))?;


    let res = FluentResource::try_new(entry.ftl.to_string()).ok()?;
    let langid: LanguageIdentifier = entry.code.parse().ok()?;
    let mut bundle = FluentBundle::new(vec![langid]);
    bundle.add_resource(res).ok()?;

    Some(bundle)
}

pub fn fl(key: &str) -> String {
    FLUENT_BUNDLE.with(|cell| {
        let bundle = cell.get_or_init(|| {
            load_fluent_bundle().expect("Could not load translation bundle")
        });

        let mut errors = vec![];
        bundle
            .get_message(key)
            .and_then(|msg| msg.value())
            .map(|pattern| bundle.format_pattern(pattern, None, &mut errors).to_string())
            .unwrap_or_else(|| format!("{{{}}}", key))
    })
}
