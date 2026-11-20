//! End-to-end behavior of the index: what a reader typing a search will see.

use mulu_index::{Hit, LibraryIndex, Page};

fn page(file_id: u64, number: u32, text: &str) -> Page {
    Page {
        file_id,
        number,
        label: format!("p. {number}"),
        text: text.to_string(),
    }
}

/// File ids in result order, and their occurrence counts.
fn summary(index: &LibraryIndex, query: &str) -> Vec<(u64, u32)> {
    index
        .search(query)
        .unwrap()
        .files
        .iter()
        .map(|f| (f.file_id, f.count))
        .collect()
}

/// All passages of the first file in the results.
fn first_file_hits(index: &LibraryIndex, query: &str) -> Vec<Hit> {
    let results = index.search(query).unwrap();
    index
        .passages(&results, &results.files[0], 0, usize::MAX)
        .unwrap()
}

fn library() -> LibraryIndex {
    let index = LibraryIndex::in_memory().unwrap();
    let mut w = index.writer(15).unwrap();
    // File 1: Dom Casmurro (public domain), two pages.
    w.add_page(&page(
        1,
        1,
        "Capitu tinha olhos de ressaca. Olhos de cigana oblíqua e dissimulada.",
    ))
    .unwrap();
    w.add_page(&page(
        1,
        2,
        "Os olhos de ressaca de Capitu traziam não sei que fluido misterioso.",
    ))
    .unwrap();
    // File 2: one page mentioning the eyes once.
    w.add_page(&page(
        2,
        1,
        "A ação se passa no Rio de Janeiro; os olhos de todos se voltam para ela.",
    ))
    .unwrap();
    // File 3: Chinese.
    w.add_page(&page(3, 1, "武汉市长江大桥。长江是中国最长的河流。"))
        .unwrap();
    // File 4: the two words on different pages.
    w.add_page(&page(4, 1, "Primeira página: Capitu.")).unwrap();
    w.add_page(&page(4, 7, "Sétima página: ressaca.")).unwrap();
    w.commit().unwrap();
    index.refresh().unwrap();
    index
}

#[test]
fn files_with_most_occurrences_come_first() {
    assert_eq!(summary(&library(), "olhos"), [(1, 3), (2, 1)]);
}

#[test]
fn occurrences_are_in_page_order_with_labels() {
    let hits = first_file_hits(&library(), "olhos");
    let pages: Vec<&str> = hits.iter().map(|h| h.label.as_str()).collect();
    assert_eq!(pages, ["p. 1", "p. 1", "p. 2"]);
}

#[test]
fn exact_phrase_between_quotes() {
    let index = library();
    assert_eq!(summary(&index, "\"olhos de ressaca\""), [(1, 2)]);
    assert_eq!(
        first_file_hits(&index, "\"olhos de ressaca\"")[0]
            .passage
            .matched,
        "olhos de ressaca"
    );
    assert!(summary(&index, "\"ressaca de olhos\"").is_empty());
}

#[test]
fn several_words_must_all_be_in_the_file_but_not_on_the_same_page() {
    let files: Vec<u64> = summary(&library(), "capitu ressaca")
        .iter()
        .map(|(f, _)| *f)
        .collect();
    assert_eq!(files, [1, 4]);
}

#[test]
fn accents_and_case_are_ignored() {
    let index = library();
    assert_eq!(first_file_hits(&index, "ACAO")[0].passage.matched, "ação");
    assert_eq!(summary(&index, "obliqua"), [(1, 1)]);
}

#[test]
fn chinese_substrings_and_single_characters() {
    let index = library();
    assert_eq!(summary(&index, "长江"), [(3, 2)]);
    assert_eq!(summary(&index, "长"), [(3, 3)]);
    assert_eq!(summary(&index, "长江大桥"), [(3, 1)]);
    assert!(summary(&index, "汉长").is_empty());
}

#[test]
fn counts_from_the_index_match_the_passages() {
    let index = library();
    for query in [
        "olhos",
        "\"olhos de\"",
        "长",
        "长江",
        "capitu ressaca",
        "de",
        "dissimul*",
        "olh*",
        "capi* ressaca",
    ] {
        let results = index.search(query).unwrap();
        for file in &results.files {
            let hits = index.passages(&results, file, 0, usize::MAX).unwrap();
            assert_eq!(
                hits.len() as u32,
                file.count,
                "{query} in file {}",
                file.file_id
            );
        }
    }
}

#[test]
fn passages_can_be_fetched_in_portions() {
    let index = library();
    let results = index.search("olhos").unwrap();
    let file = &results.files[0];
    let first = index.passages(&results, file, 0, 2).unwrap();
    let rest = index.passages(&results, file, 2, 10).unwrap();
    assert_eq!((first.len(), rest.len()), (2, 1));
    assert_eq!(rest[0].label, "p. 2");
}

#[test]
fn passage_shows_context() {
    let p = &first_file_hits(&library(), "cigana")[0].passage;
    assert_eq!(p.before, "Capitu tinha olhos de ressaca. Olhos de ");
    assert_eq!(p.after, " oblíqua e dissimulada.");
}

#[test]
fn nothing_to_search_gives_nothing() {
    assert!(summary(&library(), "  ,.; ").is_empty());
    assert!(summary(&library(), "inexistente").is_empty());
}

#[test]
fn removing_a_file_removes_all_its_pages() {
    let index = library();
    let mut w = index.writer(15).unwrap();
    w.remove_file(1);
    w.commit().unwrap();
    index.refresh().unwrap();
    assert_eq!(summary(&index, "olhos"), [(2, 1)]);
}

#[test]
fn the_index_survives_closing_and_reopening() {
    let folder = tempfile::tempdir().unwrap();
    {
        let index = LibraryIndex::open(folder.path()).unwrap();
        let mut w = index.writer(15).unwrap();
        w.add_page(&page(9, 1, "Quem conta um conto aumenta um ponto."))
            .unwrap();
        w.commit().unwrap();
    }
    let index = LibraryIndex::open(folder.path()).unwrap();
    index.refresh().unwrap();
    assert_eq!(summary(&index, "conto"), [(9, 1)]);
}

#[test]
fn words_by_their_beginning() {
    let index = library();
    // "oblíqua" and "olhos": accents ignored, every form counted.
    assert_eq!(summary(&index, "OLH*"), [(1, 3), (2, 1)]);
    assert_eq!(summary(&index, "obliq*"), [(1, 1)]);
    let hits = first_file_hits(&index, "dissimul*");
    assert_eq!(hits[0].passage.matched, "dissimulada");
    assert!(summary(&index, "zzz*").is_empty());
}
