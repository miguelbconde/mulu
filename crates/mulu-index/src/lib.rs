//! # mulu-index
//!
//! The full-text index. Each page of a book (or each chapter or section, for formats
//! without fixed pages) is stored as one entry, with the file it belongs to. A search
//! returns each file once, with every occurrence and the passage around it.
//!
//! Search rules for version 0.1 (project instructions, section 5):
//! - several words without quotes: files that contain all of them, anywhere;
//! - text between quotes: that exact phrase;
//! - case and accents are ignored; Chinese and other unspaced scripts match exactly
//!   the characters typed (decisions D13 and D15).

mod passages;
mod tokenizer;

pub use passages::{Passage, occurrences, passage};

use mulu_text::Search;
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
    /// with [`LibraryIndex::passages`].
    pub fn search(&self, query: &str) -> Result<Results> {
        let parts: Vec<Search> = split_query(query)
            .iter()
            .map(|part| mulu_text::search(part))
            .filter(|s| *s != Search::Nothing)
            .collect();
        let searcher = self.reader.searcher();
        if parts.is_empty() {
            return Ok(Results {
                parts,
                searcher,
                files: Vec::new(),
            });
        }

        // For each part, the occurrences per page, grouped by file.
        let mut per_part: Vec<HashMap<u64, Vec<PageHits>>> = Vec::new();
        for part in &parts {
            let mut by_file: HashMap<u64, Vec<PageHits>> = HashMap::new();
            for (segment_ord, segment) in searcher.segment_readers().iter().enumerate() {
                let file_ids = segment.fast_fields().u64("file_id")?;
                let pages = segment.fast_fields().u64("page")?;
                let alive = segment.alive_bitset();
                for (doc, count) in self.count_in_segment(segment, part)? {
                    if alive.is_some_and(|a| a.is_deleted(doc)) {
                        continue;
                    }
                    let (Some(file), Some(page)) = (file_ids.first(doc), pages.first(doc)) else {
                        continue;
                    };
                    by_file.entry(file).or_default().push(PageHits {
                        page: u32::try_from(page).unwrap_or(u32::MAX),
                        count,
                        address: DocAddress::new(u32::try_from(segment_ord).unwrap_or(0), doc),
                    });
                }
            }
            per_part.push(by_file);
        }

        // Files that contain every part of the query.
        let mut files: Vec<u64> = per_part[0].keys().copied().collect();
        files.retain(|f| per_part[1..].iter().all(|by_file| by_file.contains_key(f)));

        let mut results: Vec<FileResult> = files
            .into_iter()
            .map(|file_id| {
                let mut pages: BTreeMap<(u32, DocAddress), u32> = BTreeMap::new();
                for by_file in &per_part {
                    for p in &by_file[&file_id] {
                        *pages.entry((p.page, p.address)).or_default() += p.count;
                    }
                }
                let pages: Vec<PageHits> = pages
                    .into_iter()
                    .map(|((page, address), count)| PageHits {
                        page,
                        count,
                        address,
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
            parts,
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
            let mut ranges: Vec<(usize, usize)> = results
                .parts
                .iter()
                .flat_map(|part| occurrences(text, part))
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

        // Phrase: walk the documents that contain every term, then count the
        // positions where each term sits at its expected offset from the first.
        let mut positions: Vec<u32> = Vec::new();
        let mut candidates: Vec<u32> = Vec::new();
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
                let wanted: HashSet<u32> =
                    positions.iter().map(|x| x.wrapping_sub(*offset)).collect();
                candidates.retain(|c| wanted.contains(c));
                if candidates.is_empty() {
                    break;
                }
            }
            if !candidates.is_empty() {
                out.push((doc, u32::try_from(candidates.len()).unwrap_or(u32::MAX)));
            }
            doc = postings[0].1.advance();
        }
        Ok(out)
    }
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
    parts: Vec<Search>,
    searcher: tantivy::Searcher,
    pub files: Vec<FileResult>,
}

impl Results {
    /// Total number of occurrences in all files.
    pub fn total(&self) -> u64 {
        self.files.iter().map(|f| u64::from(f.count)).sum()
    }
}

/// A file that matches the search.
#[derive(Debug, Clone)]
pub struct FileResult {
    pub file_id: u64,
    /// Number of occurrences in the whole file.
    pub count: u32,
    pages: Vec<PageHits>,
}

#[derive(Debug, Clone, Copy)]
struct PageHits {
    page: u32,
    count: u32,
    address: DocAddress,
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

/// Splits what the user typed into parts: each quoted phrase is one part, and every
/// other word is a part of its own. An unclosed quote runs to the end.
pub fn split_query(query: &str) -> Vec<String> {
    let mut parts = Vec::new();
    for (i, chunk) in query.split(['"', '“', '”']).enumerate() {
        if i % 2 == 1 {
            if !chunk.trim().is_empty() {
                parts.push(chunk.trim().to_string());
            }
        } else {
            parts.extend(chunk.split_whitespace().map(String::from));
        }
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::split_query;

    #[test]
    fn query_parts() {
        assert_eq!(split_query("cidade deus"), ["cidade", "deus"]);
        assert_eq!(
            split_query("\"cidade de deus\" rio"),
            ["cidade de deus", "rio"]
        );
        assert_eq!(split_query("“olhos de ressaca”"), ["olhos de ressaca"]);
        assert_eq!(split_query("\"sem fim"), ["sem fim"]);
        assert!(split_query("  \"\" ").is_empty());
    }
}
