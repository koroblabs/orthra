//! The language a file's path says it is written in: a translation, such as `locales/de.json`.
//!
//! A translation's strings are in its language, whatever the language of the code around them. On
//! a project beside orthra, the German, Spanish and French strings of three such files were 8 800
//! findings checked as English, more than the rest of the project together (2026-11-08). A file is
//! a translation only by its path, never by guessing at its words (`CLAUDE.md` §1.2): somewhere
//! below a directory that holds translations, named by a language tag or inside a directory that
//! is; or named the way a platform names its translations wherever they are: Android's
//! `res/values-es/`, Apple's `es.lproj/`, .NET's `Strings.es.resx`, Java's
//! `messages_es.properties`. AntennaPod's 77 Android
//! translations were 59 800 findings checked as English (2026-11-09).

/// Directories that hold translations, by the names i18n libraries give them: Rails' and
/// i18next's `locales/`, Laravel's `lang/`, Chrome's `_locales/`, next-intl's `messages/`, Hugo's
/// and Angular's `i18n/`.
const DIRECTORIES: &[&str] = &[
    "_locales",
    "i18n",
    "l10n",
    "lang",
    "langs",
    "languages",
    "locale",
    "locales",
    "messages",
    "translations",
];

/// ISO 639-1: the two-letter codes that start a language tag. A file or directory below one of
/// [`DIRECTORIES`] names a language only when it starts with one, so `lang/api.json` is not one.
/// With the three that ISO withdrew and Java and Android still write: `in` for Indonesian, `iw`
/// for Hebrew, `ji` for Yiddish.
const LANGUAGES: &[&str] = &[
    "aa", "ab", "ae", "af", "ak", "am", "an", "ar", "as", "av", "ay", "az", "ba", "be", "bg", "bh",
    "bi", "bm", "bn", "bo", "br", "bs", "ca", "ce", "ch", "co", "cr", "cs", "cu", "cv", "cy", "da",
    "de", "dv", "dz", "ee", "el", "en", "eo", "es", "et", "eu", "fa", "ff", "fi", "fj", "fo", "fr",
    "fy", "ga", "gd", "gl", "gn", "gu", "gv", "ha", "he", "hi", "ho", "hr", "ht", "hu", "hy", "hz",
    "ia", "id", "ie", "ig", "ii", "ik", "in", "io", "is", "it", "iu", "iw", "ja", "ji", "jv", "ka",
    "kg", "ki", "kj", "kk", "kl", "km", "kn", "ko", "kr", "ks", "ku", "kv", "kw", "ky", "la", "lb",
    "lg", "li", "ln", "lo", "lt", "lu", "lv", "mg", "mh", "mi", "mk", "ml", "mn", "mr", "ms", "mt",
    "my", "na", "nb", "nd", "ne", "ng", "nl", "nn", "no", "nr", "nv", "ny", "oc", "oj", "om", "or",
    "os", "pa", "pi", "pl", "ps", "pt", "qu", "rm", "rn", "ro", "ru", "rw", "sa", "sc", "sd", "se",
    "sg", "si", "sk", "sl", "sm", "sn", "so", "sq", "sr", "ss", "st", "su", "sv", "sw", "ta", "te",
    "tg", "th", "ti", "tk", "tl", "tn", "to", "tr", "ts", "tt", "tw", "ty", "ug", "uk", "ur", "uz",
    "ve", "vi", "vo", "wa", "wo", "xh", "yi", "yo", "za", "zh", "zu",
];

/// The language, as its two-letter code, that the path of a translation names: `de` for
/// `web/src/i18n/de.json`, `config/locales/devise.de.yml`, `public/locales/de-DE/common.json`,
/// `res/values-de/strings.xml`, `de.lproj/Main.strings`, `Strings.de.resx` and
/// `messages_de.properties`. `None` for any other path.
pub fn named_language(path: &str) -> Option<&str> {
    let mut parts = path.split(['/', '\\']).filter(|part| !part.is_empty());
    let name = parts.next_back()?;
    // Java's resource bundles: `messages_de.properties`, `JabRef_pt_BR.properties`. JabRef's
    // were 5 000 to 6 700 findings each, checked as English (2026-11-09).
    if let Some(stem) = name.strip_suffix(".properties")
        && let Some(language) = stem
            .match_indices('_')
            .find_map(|(at, _)| language_of(&stem[at + 1..]))
    {
        return Some(language);
    }
    // .NET's satellite resources: `Strings.de.resx`, `Resources.pt-BR.resx`.
    if let Some(stem) = name.strip_suffix(".resx")
        && let Some((_, tag)) = stem.rsplit_once('.')
        && let Some(language) = language_of(tag)
    {
        return Some(language);
    }
    let mut below_translations = false;
    let mut after_translations = false;
    for directory in parts {
        if after_translations && let Some(language) = language_of(directory) {
            return Some(language);
        }
        if let Some(language) = platform_directory(directory) {
            return Some(language);
        }
        after_translations = DIRECTORIES.contains(&directory);
        below_translations |= after_translations;
    }
    if !below_translations {
        return None;
    }
    // `de.json`, `pt_BR.yml`; and the part before the extension, Rails' `devise.de.yml`.
    let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);
    language_of(stem).or_else(|| language_of(stem.rsplit_once('.')?.1))
}

/// The language a directory names the way a platform names a translation's: Android's
/// `values-de`, `values-pt-rBR`, `values-ast` and `values-b+sr+Latn`, and Apple's `de.lproj` and
/// `pt-BR.lproj`. Android's other qualifiers are not languages: `values-night`, `values-v21`,
/// `values-sw600dp`, and `values-car`, the one of three letters.
fn platform_directory(directory: &str) -> Option<&str> {
    if let Some(qualifiers) = directory.strip_prefix("values-") {
        let language = match qualifiers.strip_prefix("b+") {
            Some(tag) => tag.split('+').next()?,
            None => qualifiers.split('-').next()?,
        };
        let three = language.len() == 3
            && language.bytes().all(|b| b.is_ascii_lowercase())
            && language != "car";
        return (three || LANGUAGES.contains(&language)).then_some(language);
    }
    language_of(directory.strip_suffix(".lproj")?)
}

/// The language of `tag` when it is a language tag that starts with a two-letter code: `de`,
/// `pt-BR`, `pt_BR`, `zh-Hant`, `es-419`.
fn language_of(tag: &str) -> Option<&str> {
    let mut subtags = tag.split(['-', '_']);
    let language = subtags.next()?;
    let well_formed = subtags.all(|subtag| {
        (2..=8).contains(&subtag.len()) && subtag.bytes().all(|b| b.is_ascii_alphanumeric())
    });
    (well_formed && LANGUAGES.contains(&language)).then_some(language)
}

/// The language of the pack tag `tag`: its first subtag, `en` in `en-US`.
pub fn language(tag: &str) -> &str {
    tag.split(['-', '_']).next().unwrap_or_default()
}

#[cfg(test)]
mod tests;
