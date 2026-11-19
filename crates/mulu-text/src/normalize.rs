//! Folding: the rules that make different spellings of the same word meet in the index.

use crate::script::{is_optional_mark, marks_are_optional};
use unicode_normalization::UnicodeNormalization;

/// Folds a word (or one character of an unspaced script) to its search form.
///
/// - letter case is ignored ("Livro" = "livro"; Greek final sigma = sigma; "ß" = "ss");
/// - compatibility forms are unified (ligatures, full-width and half-width forms);
/// - accents and other optional marks are ignored in Latin, Greek, Cyrillic, Hebrew and
///   Arabic ("ação" = "acao", "λόγος" = "λογος", Arabic harakat and Hebrew niqqud removed),
///   but kept in scripts where they are part of the letters (Devanagari and other Indic scripts);
/// - common Arabic spelling variants are unified (alef forms, alef maqsura, teh marbuta,
///   Persian yeh and keheh, tatweel).
///
/// The same function runs on indexed text and on queries, so both always agree.
pub fn fold(word: &str) -> String {
    let lowered = word.to_lowercase();
    let mut out = String::with_capacity(lowered.len());
    let mut base_allows_dropping = false;
    for c in lowered.nfkd() {
        if is_optional_mark(c) && base_allows_dropping {
            continue;
        }
        if !is_optional_mark(c) {
            base_allows_dropping = marks_are_optional(c);
        }
        match c {
            'ß' => out.push_str("ss"),
            'ς' => out.push('σ'),
            // Arabic: alef wasla and other alef forms, alef maqsura, teh marbuta
            'ٱ' | 'ٲ' | 'ٳ' => out.push('ا'),
            'ى' | 'ی' => out.push('ي'),
            'ک' => out.push('ك'),
            'ة' => out.push('ه'),
            // Arabic tatweel (kashida) only stretches a word on the page
            '\u{0640}' => {}
            _ => out.push(c),
        }
    }
    out.nfc().collect()
}

#[cfg(test)]
mod tests {
    use super::fold;

    #[test]
    fn latin_accents_and_case() {
        assert_eq!(fold("AÇÃO"), "acao");
        assert_eq!(fold("Éluard"), "eluard");
        assert_eq!(fold("Straße"), "strasse");
        assert_eq!(fold("ﬁn"), "fin");
        assert_eq!(fold("ＡＢＣ"), "abc");
        assert_eq!(fold("İstanbul"), "istanbul");
    }

    #[test]
    fn greek_modern_and_ancient() {
        assert_eq!(fold("Λόγος"), "λογοσ");
        assert_eq!(fold("ἀρχῇ"), "αρχη");
        assert_eq!(fold("ΛΟΓΟΣ"), fold("λόγος"));
    }

    #[test]
    fn arabic_variants() {
        assert_eq!(fold("وَالْكِتَابُ"), "والكتاب");
        assert_eq!(fold("أحمد"), fold("احمد"));
        assert_eq!(fold("إسلام"), fold("اسلام"));
        assert_eq!(fold("مكتبة"), fold("مكتبه"));
        assert_eq!(fold("على"), fold("علي"));
        assert_eq!(fold("كــتاب"), "كتاب");
        assert_eq!(fold("کتاب"), fold("كتاب"));
    }

    #[test]
    fn hebrew_points_removed() {
        assert_eq!(fold("סֵפֶר"), "ספר");
    }

    #[test]
    fn devanagari_vowel_signs_kept() {
        assert_eq!(fold("हिन्दी"), "हिन्दी".chars().collect::<String>());
        assert_ne!(fold("हिन्दी"), fold("हनद"));
    }

    #[test]
    fn japanese_width_unified() {
        assert_eq!(fold("ｶﾞ"), "ガ");
    }
}
