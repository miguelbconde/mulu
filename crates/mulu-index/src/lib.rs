//! # mulu-index
//!
//! The full-text index. Each page of a book (or each chapter or section, for formats
//! without fixed pages) is stored as one entry, with the file it belongs to. A search
//! returns each file once, with every occurrence and the passage around it.
//!
//! Search rules (project instructions, section 5, and decision D33): several words
//! find the files that contain all of them, anywhere; quotes, `-`, `|`, `~N`, `*`,
//! brackets and `name:` are read by [`mulu_text::query`]. Whether a file matches is
//! decided file by file; occurrences are counted page by page. Case and accents are
//! ignored; Chinese and other unspaced scripts match exactly the characters typed
//! (decisions D13 and D15).

mod passages;
mod tokenizer;

pub use passages::{Passage, near_occurrences, occurrences, passage};

use mulu_text::{Query, Search};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use tantivy::directory::MmapDirectory;
use tantivy::postings::Postings;
use tantivy::schema::{
    FAST, Field, INDEXED, IndexRecordOption, STORED, Schema, TextFieldIndexing, TextOptions, Value,
};
use tantivy::{
    DocAddress, DocId, DocSet, Index, IndexReader, IndexWriter, ReloadPolicy, SegmentReader,
    TERMINATED, TantivyDocument, Term, doc,
};

pub use tantivy::TantivyError as Error;
pub type Result<T> = std::result::Result<T, Error>;

/// Passage length on each side of an occurrence, in words (project instructions).
pub const PASSAGE_WORDS: usize = 25;

/// One page, chapter or section of a file, ready to be indexed.
#[derive(Debug, Clone)]
pub struct Page {
    /// The file this page belongs to (its identifier in Mulu's catalog).
    pub file_id: u64,
    /// Order of the page inside the file, starting at 1.
    pub number: u32,
    /// How the page is shown to the user: "p. 12", "Chapter 3", or empty.
    pub label: String,
    pub text: String,
}

/// One occurrence of the search in a file.
#[derive(Debug, Clone)]
pub struct Hit {
    pub page: u32,
    pub label: String,
    pub passage: Passage,
}

#[derive(Clone, Copy)]
struct Fields {
    file_id: Field,
    page: Field,
    label: Field,
    text: Field,
    chars: Field,
}

fn schema() -> (Schema, Fields) {
    let mut builder = Schema::builder();
    let file_id = builder.add_u64_field("file_id", INDEXED | STORED | FAST);
    let page = builder.add_u64_field("page", STORED | FAST);
    let label = builder.add_text_field("label", STORED);
    let text = builder.add_text_field(
        "text",
        TextOptions::default().set_stored().set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(tokenizer::MAIN)
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        ),
    );
    let chars = builder.add_text_field(
        "chars",
        TextOptions::default().set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(tokenizer::SINGLE_CHARACTERS)
                .set_index_option(IndexRecordOption::WithFreqs),
        ),
    );
    (
        builder.build(),
        Fields {
            file_id,
            page,
            label,
            text,
            chars,
        },
    )
}

/// The index of one library, stored in its own folder.
pub struct LibraryIndex {
    index: Index,
    reader: IndexReader,
    fields: Fields,
}

/// Adds and removes pages. Changes become visible to searches after [`IndexWriterHandle::commit`].
pub struct IndexWriterHandle {
    writer: IndexWriter,
    fields: Fields,
}

impl LibraryIndex {
    /// Opens the index in `folder`, creating it if the folder is empty.
    pub fn open(folder: &Path) -> Result<Self> {
        std::fs::create_dir_all(folder).map_err(|e| Error::SystemError(e.to_string()))?;
        let (schema, fields) = schema();
        let directory = MmapDirectory::open(folder)?;
        let index = Index::open_or_create(directory, schema)?;
        Self::finish(index, fields)
    }

    /// An index kept only in memory, for tests.
    pub fn in_memory() -> Result<Self> {
        let (schema, fields) = schema();
        Self::finish(Index::create_in_ram(schema), fields)
    }

    fn finish(index: Index, fields: Fields) -> Result<Self> {
        index
            .tokenizers()
            .register(tokenizer::MAIN, tokenizer::MuluTokenizer::main());
        index.tokenizers().register(
            tokenizer::SINGLE_CHARACTERS,
            tokenizer::MuluTokenizer::single_characters(),
        );
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;
        Ok(Self {
            index,
            reader,
            fields,
        })
    }

    /// A writer that uses at most `memory_mb` megabytes of memory while indexing.
    pub fn writer(&self, memory_mb: usize) -> Result<IndexWriterHandle> {
        let writer = self
            .index
            .writer_with_num_threads(1, memory_mb.max(15) * 1_000_000)?;
        Ok(IndexWriterHandle {
            writer,
            fields: self.fields,
        })
    }

    /// Makes the latest committed changes visible to searches.
    pub fn refresh(&self) -> Result<()> {
        self.reader.reload()
    }

    /// Runs a search. Every matching file is returned once, with its number of
    /// occurrences, the files with the most occurrences first. Counting uses only the
    /// index, so it stays fast for common words; passages are cut later, on demand,
    /// with [`LibraryIndex::passages`]. Parts asked for in file names (`name:`) find
    /// nothing here; see [`LibraryIndex::search_with_names`].
    pub fn search(&self, query: &str) -> Result<Results> {
        self.search_with_names(query, &|_| HashSet::new())
    }

    /// Runs a search; `names` answers the parts asked for in file names (`name:`),
    /// which only the catalog knows, with the identifiers of the matching files.
    /// Files found only through names come back with no occurrences.
    pub fn search_with_names(
        &self,
        query: &str,
        names: &dyn Fn(&Query) -> HashSet<u64>,
    ) -> Result<Results> {
        let tree = mulu_text::parse(query);
        let searcher = self.reader.searcher();
        let mut run = Run {
            index: self,
            searcher: &searcher,
            names,
            leaves: Vec::new(),
        };
        let found = if tree.is_empty() {
            FileHits::new()
        } else {
            run.eval(&tree)?
        };
        let leaves = run.leaves;

        let mut results: Vec<FileResult> = found
            .into_iter()
            .map(|(file_id, hits)| {
                let mut pages: BTreeMap<(u32, DocAddress), (u32, Vec<u16>)> = BTreeMap::new();
                for h in hits {
                    let entry = pages.entry((h.page, h.address)).or_default();
                    entry.0 += h.count;
                    if !entry.1.contains(&h.leaf) {
                        entry.1.push(h.leaf);
                    }
                }
                let pages: Vec<PageHits> = pages
                    .into_iter()
                    .map(|((page, address), (count, leaves))| PageHits {
                        page,
                        count,
                        address,
                        leaves,
                    })
                    .collect();
                let count = pages.iter().map(|p| p.count).sum();
                FileResult {
                    file_id,
                    count,
                    pages,
                }
            })
            .collect();
        results.sort_by(|a, b| b.count.cmp(&a.count).then(a.file_id.cmp(&b.file_id)));
        Ok(Results {
            leaves,
            searcher,
            files: results,
        })
    }

    /// Cuts the passages of one file, in page order, skipping the first `skip`
    /// occurrences and returning at most `limit`.
    pub fn passages(
        &self,
        results: &Results,
        file: &FileResult,
        skip: usize,
        limit: usize,
    ) -> Result<Vec<Hit>> {
        let mut out = Vec::new();
        let mut seen = 0usize;
        for page in &file.pages {
            if out.len() >= limit {
                break;
            }
            if seen + page.count as usize <= skip {
                seen += page.count as usize;
                continue;
            }
            let document: TantivyDocument = results.searcher.doc(page.address)?;
            let text = document
                .get_first(self.fields.text)
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let label = document
                .get_first(self.fields.label)
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let mut ranges: Vec<(usize, usize)> = page
                .leaves
                .iter()
                .filter_map(|&leaf| results.leaves.get(usize::from(leaf)))
                .flat_map(|leaf| match leaf {
                    Leaf::Term(search) => occurrences(text, search),
                    Leaf::Near(terms, within) => near_occurrences(text, terms, *within),
                })
                .collect();
            ranges.sort_unstable();
            for (start, end) in ranges {
                if seen >= skip && out.len() < limit {
                    out.push(Hit {
                        page: page.page,
                        label: label.to_string(),
                        passage: passage(text, start, end, PASSAGE_WORDS),
                    });
                }
                seen += 1;
            }
        }
        Ok(out)
    }

    /// Occurrences of one query part in each page of a segment, read from the
    /// postings lists only (no stored text).
    fn count_in_segment(
        &self,
        segment: &SegmentReader,
        part: &Search,
    ) -> Result<Vec<(DocId, u32)>> {
        let (field, terms): (Field, Vec<(u32, String)>) = match part {
            Search::SingleCharacter(c) => (self.fields.chars, vec![(0, c.clone())]),
            Search::Phrase(terms) => (self.fields.text, terms.clone()),
            Search::Prefix(prefix) => return self.count_prefix(segment, prefix),
            Search::Nothing => return Ok(Vec::new()),
        };
        let inverted = segment.inverted_index(field)?;
        let option = if terms.len() == 1 {
            IndexRecordOption::WithFreqs
        } else {
            IndexRecordOption::WithFreqsAndPositions
        };
        let mut postings = Vec::new();
        for (offset, text) in &terms {
            match inverted.read_postings(&Term::from_field_text(field, text), option)? {
                Some(p) => postings.push((*offset, p)),
                None => return Ok(Vec::new()),
            }
        }

        let mut out = Vec::new();
        if postings.len() == 1 {
            let p = &mut postings[0].1;
            while p.doc() != TERMINATED {
                out.push((p.doc(), p.term_freq()));
                p.advance();
            }
            return Ok(out);
        }

        Ok(phrase_positions(postings)
            .into_iter()
            .map(|(doc, at)| (doc, u32::try_from(at.len()).unwrap_or(u32::MAX)))
            .collect())
    }

    /// Where a word or phrase starts in each page of a segment, as word positions.
    fn positions_in_segment(
        &self,
        segment: &SegmentReader,
        terms: &[(u32, String)],
    ) -> Result<Vec<(DocId, Vec<u32>)>> {
        let inverted = segment.inverted_index(self.fields.text)?;
        let mut postings = Vec::new();
        for (offset, text) in terms {
            match inverted.read_postings(
                &Term::from_field_text(self.fields.text, text),
                IndexRecordOption::WithFreqsAndPositions,
            )? {
                Some(p) => postings.push((*offset, p)),
                None => return Ok(Vec::new()),
            }
        }
        Ok(phrase_positions(postings))
    }

    /// Occurrences of several terms near each other in each page of a segment: an
    /// occurrence counts when every other term has one within `within` words of it.
    fn count_near(
        &self,
        segment: &SegmentReader,
        terms: &[Search],
        within: u32,
    ) -> Result<Vec<(DocId, u32)>> {
        let mut per_term: Vec<HashMap<DocId, Vec<u32>>> = Vec::new();
        for term in terms {
            let Search::Phrase(words) = term else {
                return Ok(Vec::new());
            };
            per_term.push(
                self.positions_in_segment(segment, words)?
                    .into_iter()
                    .collect(),
            );
        }
        let Some((first, rest)) = per_term.split_first() else {
            return Ok(Vec::new());
        };
        let mut out = Vec::new();
        for doc in first.keys() {
            let lists: Option<Vec<&Vec<u32>>> = std::iter::once(first.get(doc))
                .chain(rest.iter().map(|m| m.get(doc)))
                .collect();
            let Some(lists) = lists else { continue };
            let count = near_count(&lists, within);
            if count > 0 {
                out.push((*doc, count));
            }
        }
        out.sort_unstable();
        Ok(out)
    }
}

/// Walks the pages that contain every term of a phrase and returns, for each, the
/// positions where each term sits at its expected offset from the first.
fn phrase_positions(
    mut postings: Vec<(u32, tantivy::postings::SegmentPostings)>,
) -> Vec<(DocId, Vec<u32>)> {
    let mut out = Vec::new();
    let mut positions: Vec<u32> = Vec::new();
    let mut candidates: Vec<u32> = Vec::new();
    if postings.is_empty() {
        return out;
    }
    let mut doc = postings[0].1.doc();
    'docs: while doc != TERMINATED {
        for (_, p) in postings.iter_mut() {
            let found = p.seek(doc);
            if found != doc {
                doc = found;
                continue 'docs;
            }
        }
        let (first_offset, first) = &mut postings[0];
        let first_offset = *first_offset;
        first.positions(&mut candidates);
        for c in candidates.iter_mut() {
            *c = c.wrapping_sub(first_offset);
        }
        for (offset, p) in postings[1..].iter_mut() {
            p.positions(&mut positions);
            let wanted: HashSet<u32> = positions.iter().map(|x| x.wrapping_sub(*offset)).collect();
            candidates.retain(|c| wanted.contains(c));
            if candidates.is_empty() {
                break;
            }
        }
        if !candidates.is_empty() {
            out.push((doc, candidates.clone()));
        }
        doc = postings[0].1.advance();
    }
    out
}

/// How many occurrences, among several lists of positions (each sorted), have an
/// occurrence of every other list within `within` positions. The same rule cuts the
/// passages ([`near_occurrences`]).
pub(crate) fn near_count(lists: &[&Vec<u32>], within: u32) -> u32 {
    let mut count = 0u32;
    for (i, list) in lists.iter().enumerate() {
        for &p in list.iter() {
            let close = lists
                .iter()
                .enumerate()
                .all(|(j, other)| j == i || has_near(other, p, within));
            if close {
                count += 1;
            }
        }
    }
    count
}

/// Whether the sorted list has a position within `within` of `p`.
pub(crate) fn has_near(sorted: &[u32], p: u32, within: u32) -> bool {
    let from = p.saturating_sub(within);
    let at = sorted.partition_point(|&q| q < from);
    sorted
        .get(at)
        .is_some_and(|&q| q <= p.saturating_add(within))
}

impl LibraryIndex {
    /// Occurrences of every word that starts with `prefix`, per page: the term
    /// dictionary is sorted, so the matching words are read in one stretch.
    fn count_prefix(&self, segment: &SegmentReader, prefix: &str) -> Result<Vec<(DocId, u32)>> {
        let inverted = segment.inverted_index(self.fields.text)?;
        let mut counts: BTreeMap<DocId, u32> = BTreeMap::new();
        let mut stream = inverted
            .terms()
            .range()
            .ge(prefix.as_bytes())
            .into_stream()?;
        while stream.advance() {
            if !stream.key().starts_with(prefix.as_bytes()) {
                break;
            }
            let mut postings = inverted
                .read_postings_from_terminfo(stream.value(), IndexRecordOption::WithFreqs)?;
            while postings.doc() != TERMINATED {
                *counts.entry(postings.doc()).or_default() += postings.term_freq();
                postings.advance();
            }
        }
        Ok(counts.into_iter().collect())
    }
}

/// The outcome of a search, valid until the next one.
pub struct Results {
    /// What was looked for in the text, numbered as in [`PageHits::leaves`].
    leaves: Vec<Leaf>,
    searcher: tantivy::Searcher,
    pub files: Vec<FileResult>,
}

impl Results {
    /// Total number of occurrences in all files.
    pub fn total(&self) -> u64 {
        self.files.iter().map(|f| u64::from(f.count)).sum()
    }
}

/// One thing looked for in the text, whose occurrences are counted and shown.
#[derive(Debug, Clone)]
enum Leaf {
    Term(Search),
    Near(Vec<Search>, u32),
}

/// A file that matches the search.
#[derive(Debug, Clone)]
pub struct FileResult {
    pub file_id: u64,
    /// Number of occurrences in the whole file; 0 when only the name matched.
    pub count: u32,
    pages: Vec<PageHits>,
}

/// The occurrences in one page, and which leaves of the search they come from.
#[derive(Debug, Clone)]
struct PageHits {
    page: u32,
    count: u32,
    address: DocAddress,
    leaves: Vec<u16>,
}

/// Occurrences of one leaf in one page.
#[derive(Debug, Clone, Copy)]
struct LeafHits {
    page: u32,
    count: u32,
    address: DocAddress,
    leaf: u16,
}

/// For each matching file, its occurrences.
type FileHits = HashMap<u64, Vec<LeafHits>>;

/// One search being evaluated.
struct Run<'a> {
    index: &'a LibraryIndex,
    searcher: &'a tantivy::Searcher,
    names: &'a dyn Fn(&Query) -> HashSet<u64>,
    leaves: Vec<Leaf>,
}

impl Run<'_> {
    fn eval(&mut self, query: &Query) -> Result<FileHits> {
        match query {
            Query::Term(search) => self.leaf(Leaf::Term(search.clone())),
            Query::Near(terms, within) => self.leaf(Leaf::Near(terms.clone(), *within)),
            Query::Name(inner) => Ok((self.names)(inner)
                .into_iter()
                .map(|f| (f, Vec::new()))
                .collect()),
            // "Without" alone asks for nothing to be found.
            Query::Not(_) => Ok(FileHits::new()),
            Query::All(parts) => {
                let mut found: Option<FileHits> = None;
                for part in parts.iter().filter(|p| !matches!(p, Query::Not(_))) {
                    let hits = self.eval(part)?;
                    found = Some(match found {
                        None => hits,
                        Some(mut so_far) => {
                            so_far.retain(|f, _| hits.contains_key(f));
                            for (f, more) in hits {
                                if let Some(list) = so_far.get_mut(&f) {
                                    list.extend(more);
                                }
                            }
                            so_far
                        }
                    });
                    if found.as_ref().is_some_and(HashMap::is_empty) {
                        return Ok(FileHits::new());
                    }
                }
                let mut found = found.unwrap_or_default();
                for part in parts {
                    if let Query::Not(inner) = part {
                        let leaves = self.leaves.len();
                        let out = self.eval(inner)?;
                        // What is excluded is never shown.
                        self.leaves.truncate(leaves);
                        found.retain(|f, _| !out.contains_key(f));
                    }
                }
                Ok(found)
            }
            Query::Any(parts) => {
                let mut found = FileHits::new();
                for part in parts.iter().filter(|p| !matches!(p, Query::Not(_))) {
                    for (f, more) in self.eval(part)? {
                        found.entry(f).or_default().extend(more);
                    }
                }
                Ok(found)
            }
        }
    }

    fn leaf(&mut self, leaf: Leaf) -> Result<FileHits> {
        let id = u16::try_from(self.leaves.len()).unwrap_or(u16::MAX);
        let mut by_file = FileHits::new();
        for (segment_ord, segment) in self.searcher.segment_readers().iter().enumerate() {
            let file_ids = segment.fast_fields().u64("file_id")?;
            let pages = segment.fast_fields().u64("page")?;
            let alive = segment.alive_bitset();
            let counts = match &leaf {
                Leaf::Term(search) => self.index.count_in_segment(segment, search)?,
                Leaf::Near(terms, within) => self.index.count_near(segment, terms, *within)?,
            };
            for (doc, count) in counts {
                if alive.is_some_and(|a| a.is_deleted(doc)) {
                    continue;
                }
                let (Some(file), Some(page)) = (file_ids.first(doc), pages.first(doc)) else {
                    continue;
                };
                by_file.entry(file).or_default().push(LeafHits {
                    page: u32::try_from(page).unwrap_or(u32::MAX),
                    count,
                    address: DocAddress::new(u32::try_from(segment_ord).unwrap_or(0), doc),
                    leaf: id,
                });
            }
        }
        self.leaves.push(leaf);
        Ok(by_file)
    }
}

impl IndexWriterHandle {
    pub fn add_page(&mut self, page: &Page) -> Result<()> {
        let f = self.fields;
        self.writer.add_document(doc!(
            f.file_id => page.file_id,
            f.page => u64::from(page.number),
            f.label => page.label.as_str(),
            f.text => page.text.as_str(),
            f.chars => page.text.as_str(),
        ))?;
        Ok(())
    }

    /// Removes every page of a file (when it is deleted or before it is indexed again).
    pub fn remove_file(&mut self, file_id: u64) {
        self.writer
            .delete_term(Term::from_field_u64(self.fields.file_id, file_id));
    }

    /// Saves the changes made since the last commit.
    pub fn commit(&mut self) -> Result<()> {
        self.writer.commit()?;
        Ok(())
    }
}
