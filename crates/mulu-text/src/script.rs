//! Small, explicit script tables.
//!
//! Only two questions are asked about a character:
//! - is it written without spaces between words (Chinese, Japanese, Korean, Thai, ...)?
//! - may its accents and vowel signs be ignored when searching?
//!
//! A full Unicode script database would answer both, but these tables are short,
//! reviewable, and cover the scripts Mulu indexes differently from the default.

/// Scripts written without spaces between words. Text in these scripts is indexed
/// as overlapping pairs of characters (decision D13, extended to all such scripts by D15).
pub fn is_unspaced(c: char) -> bool {
    matches!(c as u32,
        // Han (CJK unified ideographs, extensions, compatibility ideographs)
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0x20000..=0x323AF
        // CJK iteration and ideographic marks: 々 〆 〇
        | 0x3005..=0x3007
        // Hiragana, Katakana, Katakana phonetic extensions, halfwidth Katakana
        | 0x3040..=0x309F | 0x30A0..=0x30FF | 0x31F0..=0x31FF | 0xFF66..=0xFF9F
        // Hangul syllables and jamo
        | 0xAC00..=0xD7AF | 0x1100..=0x11FF | 0x3130..=0x318F | 0xA960..=0xA97F | 0xD7B0..=0xD7FF
        // Thai, Lao, Myanmar, Khmer
        | 0x0E00..=0x0E7F | 0x0E80..=0x0EFF | 0x1000..=0x109F | 0x1780..=0x17FF
    )
}

/// Scripts whose combining marks (accents, breathings, vowel points) are ignored
/// when searching, the way "é" matches "e". Indic scripts are deliberately absent:
/// their vowel signs and virama are part of the letters, not optional accents.
pub fn marks_are_optional(c: char) -> bool {
    matches!(c as u32,
        // Latin
        0x0041..=0x005A | 0x0061..=0x007A | 0x00C0..=0x024F | 0x1E00..=0x1EFF
        | 0x2C60..=0x2C7F | 0xA720..=0xA7FF | 0xAB30..=0xAB6F
        // Greek (modern and polytonic)
        | 0x0370..=0x03FF | 0x1F00..=0x1FFF
        // Cyrillic
        | 0x0400..=0x052F | 0x1C80..=0x1C8F | 0x2DE0..=0x2DFF | 0xA640..=0xA69F
        // Hebrew
        | 0x0590..=0x05FF | 0xFB1D..=0xFB4F
        // Arabic
        | 0x0600..=0x06FF | 0x0750..=0x077F | 0x0870..=0x08FF | 0xFB50..=0xFDFF | 0xFE70..=0xFEFF
    )
}

/// Combining marks (Unicode general category Mn) in the ranges used by the scripts
/// in [`marks_are_optional`]. After canonical decomposition these are the accents,
/// tonos, breathings, Hebrew niqqud and Arabic harakat.
pub fn is_optional_mark(c: char) -> bool {
    matches!(c as u32,
        0x0300..=0x036F           // combining diacritical marks
        | 0x1AB0..=0x1AFF         // combining diacritical marks extended
        | 0x1DC0..=0x1DFF         // combining diacritical marks supplement
        | 0x20D0..=0x20FF         // combining marks for symbols
        | 0xFE20..=0xFE2F         // combining half marks
        | 0x0483..=0x0489         // Cyrillic combining marks
        | 0x0591..=0x05BD | 0x05BF | 0x05C1..=0x05C2 | 0x05C4..=0x05C5 | 0x05C7 // Hebrew points
        | 0x0610..=0x061A | 0x064B..=0x065F | 0x0670 | 0x06D6..=0x06DC
        | 0x06DF..=0x06E4 | 0x06E7..=0x06E8 | 0x06EA..=0x06ED | 0x08D3..=0x08E1
        | 0x08E3..=0x08FF        // Arabic harakat and Quranic marks
    )
}
