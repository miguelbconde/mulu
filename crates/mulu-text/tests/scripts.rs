//! Behavior across writing systems, checked the way the index will use it:
//! a search "finds" a document when its terms appear at the right positions.

use mulu_text::{Search, index_terms, search, single_character_terms};

fn finds(document: &str, query: &str) -> bool {
    match search(query) {
        Search::Nothing => false,
        Search::SingleCharacter(c) => single_character_terms(document).iter().any(|t| t.text == c),
        Search::Phrase(phrase) => {
            let terms = index_terms(document);
            let (_, first) = &phrase[0];
            terms.iter().filter(|t| &t.text == first).any(|anchor| {
                phrase.iter().all(|(offset, text)| {
                    terms
                        .iter()
                        .any(|t| t.position == anchor.position + offset && &t.text == text)
                })
            })
        }
    }
}

#[test]
fn portuguese_accents_case_and_phrases() {
    let doc = "A AÇÃO do romance se passa em São Paulo.";
    assert!(finds(doc, "acao"));
    assert!(finds(doc, "Ação"));
    assert!(finds(doc, "sao paulo"));
    assert!(finds(doc, "do romance"));
    assert!(!finds(doc, "romance do"));
}

#[test]
fn french_elision() {
    let doc = "L’homme révolté et l'étranger";
    assert!(finds(doc, "homme"));
    assert!(finds(doc, "etranger"));
    assert!(finds(doc, "revolte"));
}

#[test]
fn german_sharp_s() {
    assert!(finds("Die Straße", "strasse"));
    assert!(finds("Die STRASSE", "straße"));
}

#[test]
fn greek_modern_and_polytonic() {
    let doc = "Ἐν ἀρχῇ ἦν ὁ λόγος";
    assert!(finds(doc, "λογος"));
    assert!(finds(doc, "ΛΟΓΟΣ"));
    assert!(finds(doc, "εν αρχη"));
}

#[test]
fn russian() {
    assert!(finds("Война и мир", "МИР"));
    assert!(finds("Война и мир", "война и мир"));
}

#[test]
fn chinese_simplified_and_traditional() {
    let doc = "武汉市长江大桥";
    assert!(finds(doc, "长江大桥"));
    assert!(finds(doc, "大桥"));
    assert!(finds(doc, "江"));
    assert!(!finds(doc, "武汉长江"));
    assert!(finds("國立中央圖書館", "圖書館"));
    assert!(
        !finds("國立中央圖書館", "图书馆"),
        "simplified and traditional are different characters"
    );
}

#[test]
fn chinese_never_matches_across_punctuation() {
    assert!(!finds("大桥，桥梁", "大桥梁"));
    assert!(!finds("长江。大桥", "江大"));
    assert!(finds("长江。大桥", "大桥"));
}

#[test]
fn japanese_kanji_and_kana() {
    let doc = "図書館で本を読みました。";
    assert!(finds(doc, "図書館"));
    assert!(finds(doc, "本を読"));
    assert!(finds(doc, "読みました"));
    assert!(finds(doc, "本"));
    assert!(finds("ｶﾞｲﾄﾞﾌﾞｯｸ", "ガイドブック"), "half-width katakana");
}

#[test]
fn korean() {
    let doc = "한국어를 배운다";
    assert!(finds(doc, "한국어"));
    assert!(finds(doc, "배운"));
    assert!(!finds(doc, "어배"), "a space separates the two words");
}

#[test]
fn thai() {
    let doc = "ห้องสมุดดิจิทัลของฉัน";
    assert!(finds(doc, "ห้องสมุด"));
    assert!(finds(doc, "ดิจิทัล"));
}

#[test]
fn arabic() {
    let doc = "قرأتُ الكِتابَ في المكتبة";
    assert!(finds(doc, "الكتاب"));
    assert!(finds(doc, "قرات"), "hamza forms of alef are unified");
    assert!(finds(doc, "المكتبه"), "teh marbuta is unified with heh");
    // Known limit, planned for 0.8: attached prefixes such as al- are not removed yet,
    // so the bare word is not found inside "al-maktaba".
    assert!(!finds(doc, "مكتبة"));
}

#[test]
fn hebrew() {
    assert!(finds("בְּרֵאשִׁית בָּרָא", "בראשית"));
}

#[test]
fn hindi_words_stay_whole() {
    let doc = "हिन्दी साहित्य का इतिहास";
    assert!(finds(doc, "हिन्दी"));
    assert!(finds(doc, "साहित्य का इतिहास"));
    assert!(!finds(doc, "हिन"), "a word must not be cut at the virama");
}

#[test]
fn mixed_scripts() {
    let doc = "Mulu目录 version 2, 1999";
    assert!(finds(doc, "目录"));
    assert!(finds(doc, "mulu"));
    assert!(finds(doc, "1999"));
}

#[test]
fn nothing_searchable() {
    assert_eq!(search("  ,.;!  "), Search::Nothing);
    assert_eq!(search("📚"), Search::Nothing);
}

#[test]
fn offsets_point_back_to_the_original_text() {
    let doc = "L’ÉTRANGER, 武汉市 ｶﾞｲﾄﾞ";
    for t in index_terms(doc) {
        let original = &doc[t.start..t.end];
        assert_eq!(
            mulu_text::fold(&original.replace(['’', '\''], "")),
            t.text.replace(['’', '\''], "")
        );
    }
    let terms: Vec<_> = index_terms(doc).into_iter().map(|t| t.text).collect();
    assert_eq!(terms, ["l", "etranger", "武汉", "汉市", "ガイ", "イド"]);
}

/// Any input, however strange, must give terms that point inside the text and never panic.
#[test]
fn arbitrary_text_is_handled_safely() {
    let pieces = [
        "a", "É", "\u{0301}", "ß", "λ", "ῷ", "я", "ש", "\u{05B0}", "ك", "\u{064E}", "ـ", "ह",
        "\u{094D}", "中", "の", "ｶ", "\u{FF9E}", "한", "\u{1100}", "ก", "\u{0E49}", "1", " ", ",",
        "。", "’", "'", "\u{200D}", "👩", "\u{FE0F}", "\n", "\u{0000}", "\u{FFFD}", "𠀋", "ﬁ",
    ];
    let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
    for _ in 0..5_000 {
        let mut text = String::new();
        for _ in 0..(seed % 24) {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            text.push_str(pieces[usize::try_from(seed % pieces.len() as u64).unwrap()]);
        }
        let terms = index_terms(&text);
        let mut last = 0;
        for t in &terms {
            assert!(!t.text.is_empty());
            assert!(t.start < t.end && t.end <= text.len());
            assert!(text.is_char_boundary(t.start) && text.is_char_boundary(t.end));
            assert!(t.position >= last);
            last = t.position;
        }
        let _ = single_character_terms(&text);
        let _ = search(&text);
    }
}
