//! Rough timing on a corpus given as one page per line (not a benchmark yet).
use mulu_index::{LibraryIndex, Page};
use std::time::Instant;

fn main() {
    let path = std::env::args().nth(1).expect("corpus file");
    let reps: u64 = std::env::args().nth(2).map_or(10, |s| s.parse().unwrap());
    let lines: Vec<String> = std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(String::from)
        .collect();
    let dir = tempfile::tempdir().unwrap();
    let index = LibraryIndex::open(dir.path()).unwrap();
    let t = Instant::now();
    let mut w = index.writer(200).unwrap();
    let mut pages = 0;
    for r in 0..reps {
        for (i, chunk) in lines.chunks(10).enumerate() {
            let file_id = r * 1000 + i as u64 / 20;
            w.add_page(&Page {
                file_id,
                number: (i % 20 + 1) as u32,
                label: String::new(),
                text: chunk.join("\n"),
            })
            .unwrap();
            pages += 1;
        }
    }
    w.commit().unwrap();
    index.refresh().unwrap();
    println!(
        "{pages} pages indexed in {:.1} s",
        t.elapsed().as_secs_f64()
    );
    for q in ["的", "方鸿渐", "\"方鸿渐笑道\"", "鸿渐 柔嘉", "zzz"] {
        let t = Instant::now();
        let r = index.search(q).unwrap();
        let counted = t.elapsed().as_secs_f64() * 1000.0;
        // What the screen shows first: three passages for each of the first 50 files.
        for file in r.files.iter().take(50) {
            index.passages(&r, file, 0, 3).unwrap();
        }
        println!(
            "{q:>14}: {} files, {} hits, counted in {counted:.0} ms, first screen in {:.0} ms",
            r.files.len(),
            r.total(),
            t.elapsed().as_secs_f64() * 1000.0
        );
    }
}
