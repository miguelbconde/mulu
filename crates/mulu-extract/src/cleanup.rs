//! Clean-up of text taken from page layouts (PDF), measured in decision D24:
//!
//! - a word split by a hyphen at the end of a line is joined (`liber-` + `dade`), when
//!   the next line starts with a lowercase letter, so that searches find the word;
//! - a line break between two characters of a script written without spaces (Chinese,
//!   Japanese, Thai…) is removed, since it does not separate words there;
//! - an accent that old TeX fonts (OT1 encoding) draw as a separate character before
//!   its letter is put back on the letter (`Na¸c˜oes` becomes `Nações`).

use unicode_normalization::UnicodeNormalization;

pub fn tidy(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if (c == '-' || c == '\u{00AD}')
            && i > 0
            && chars[i - 1].is_alphabetic()
            && let Some(next) = after_line_break(&chars, i + 1)
            && chars[next].is_lowercase()
        {
            i = next;
            continue;
        }
        if let Some(combining) = combining_accent(c)
            && let Some(&letter) = chars.get(i + 1)
            && letter.is_alphabetic()
        {
            out.extend([undotted(letter), combining].into_iter().nfc());
            i += 2;
            continue;
        }
        // Old TeX fonts also put accents on a dotless i: "ı́" is an accented "i".
        if c == 'ı'
            && chars
                .get(i + 1)
                .is_some_and(|m| ('\u{0300}'..='\u{036F}').contains(m))
        {
            out.extend(['i', chars[i + 1]].into_iter().nfc());
            i += 2;
            continue;
        }
        if c == '\n' {
            let previous = out.chars().last();
            let mut k = i + 1;
            while k < chars.len() && matches!(chars[k], '\n' | ' ') {
                k += 1;
            }
            if previous.is_some_and(mulu_text::is_unspaced)
                && chars.get(k).copied().is_some_and(mulu_text::is_unspaced)
            {
                i = k;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

/// If `chars[from..]` is optional spaces, a line break and optional spaces, the index
/// of the first character after them.
fn after_line_break(chars: &[char], from: usize) -> Option<usize> {
    let mut j = from;
    while j < chars.len() && chars[j] == ' ' {
        j += 1;
    }
    if chars.get(j) != Some(&'\n') {
        return None;
    }
    j += 1;
    while j < chars.len() && chars[j] == ' ' {
        j += 1;
    }
    (j < chars.len()).then_some(j)
}

/// Accents on a dotless i or j are drawn on the dotless letter in old TeX fonts.
fn undotted(letter: char) -> char {
    match letter {
        'ı' => 'i',
        'ȷ' => 'j',
        other => other,
    }
}

fn combining_accent(c: char) -> Option<char> {
    Some(match c {
        '\u{00B4}' => '\u{0301}', // acute
        '\u{02DC}' => '\u{0303}', // tilde
        '\u{02C6}' => '\u{0302}', // circumflex
        '\u{00A8}' => '\u{0308}', // diaeresis
        '\u{00B8}' => '\u{0327}', // cedilla
        '\u{02C7}' => '\u{030C}', // caron
        '\u{02D8}' => '\u{0306}', // breve
        '\u{02DA}' => '\u{030A}', // ring
        '\u{02D9}' => '\u{0307}', // dot above
        '\u{00AF}' => '\u{0304}', // macron
        '\u{02DD}' => '\u{030B}', // double acute
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::tidy;

    #[test]
    fn hyphenated_words_are_joined() {
        assert_eq!(tidy("a liber-\ndade de"), "a liberdade de");
        assert_eq!(tidy("a liber- \n  dade"), "a liberdade");
        // A real hyphen before a capital letter, or not at a line end, is kept.
        assert_eq!(tidy("o Ex-\nPresidente"), "o Ex-\nPresidente");
        assert_eq!(tidy("guarda-chuva"), "guarda-chuva");
        assert_eq!(tidy("fim -\nde"), "fim -\nde");
    }

    #[test]
    fn lines_in_unspaced_scripts_are_joined() {
        assert_eq!(tidy("人人生而\n自由"), "人人生而自由");
        assert_eq!(tidy("fim\nde linha"), "fim\nde linha");
        assert_eq!(tidy("自由\nlibre"), "自由\nlibre");
    }

    #[test]
    fn separate_accents_are_put_back() {
        assert_eq!(
            tidy("Na\u{b8}c\u{2dc}oes, f\u{b4}e, \u{b4}Epoca"),
            "Nações, fé, Época"
        );
        assert_eq!(tidy("um acento \u{b4} solto"), "um acento \u{b4} solto");
        assert_eq!(
            tidy("fam\u{b4}\u{131}lia, fam\u{131}\u{301}lia"),
            "família, família"
        );
    }
}
