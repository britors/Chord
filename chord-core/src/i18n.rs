//! Translation catalog for `lyra-chord`, backed by `gettext` (PROMPT-CHORD.md §1.2:
//! pt-BR first, ready for other locales).

use std::path::Path;

const TEXT_DOMAIN: &str = "lyra-chord";

/// Binds the `lyra-chord` gettext domain to `locale_dir` (typically the install
/// prefix's `share/locale`) and activates it for the process locale. Call this once,
/// early in `main()`, before any UI is built.
pub fn init(locale_dir: impl AsRef<Path>) {
    let _ = gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, "");
    let _ = gettextrs::bindtextdomain(TEXT_DOMAIN, locale_dir.as_ref());
    let _ = gettextrs::bind_textdomain_codeset(TEXT_DOMAIN, "UTF-8");
    let _ = gettextrs::textdomain(TEXT_DOMAIN);
}

/// Translates `msgid` through the `lyra-chord` catalog, returning `msgid` itself if no
/// translation is available.
pub fn tr(msgid: &str) -> String {
    gettextrs::gettext(msgid)
}
