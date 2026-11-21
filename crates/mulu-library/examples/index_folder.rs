//! Measures the indexing of a folder: `cargo run --release --example index_folder FOLDER`
//! (the reading process is `mulu-library-worker`, built with it).

use mulu_library::Library;
use std::time::Instant;

fn main() {
    let folder = std::env::args().nth(1).expect("folder");
    let store = tempfile::tempdir().unwrap();
    let worker = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mulu-library-worker");
    let library = Library::open(folder.as_ref(), store.path()).unwrap();
    let start = Instant::now();
    let total = library.scan().unwrap();
    println!("scan: {total} files in {:.2?}", start.elapsed());
    let workers: usize = std::env::args()
        .nth(2)
        .and_then(|n| n.parse().ok())
        .unwrap_or(1);
    let readers = (0..workers)
        .map(|_| {
            mulu_reader::Reader::new(
                &worker,
                mulu_reader::Settings {
                    pdfium: std::env::var_os("MULU_PDFIUM").map(Into::into),
                    // Optional third argument: the time limit without progress, in seconds.
                    time_limit: std::env::args()
                        .nth(3)
                        .and_then(|s| s.parse().ok())
                        .map_or(mulu_reader::TIME_LIMIT, std::time::Duration::from_secs_f64),
                    ..mulu_reader::Settings::default()
                },
            )
        })
        .collect();
    let start = Instant::now();
    let mut last = Instant::now();
    let mut slow = Vec::new();
    let mut current = String::new();
    library
        .index_pending(readers, |p| {
            let spent = last.elapsed();
            if spent.as_millis() > 300 && !current.is_empty() {
                slow.push((spent, current.clone()));
            }
            current = p.current.clone().unwrap_or_default();
            last = Instant::now();
            true
        })
        .unwrap();
    println!("index: {:.2?}", start.elapsed());
    slow.sort_by_key(|s| std::cmp::Reverse(s.0));
    for (t, name) in slow.iter().take(15) {
        println!("{t:>10.2?}  {name}");
    }
    let problems = library.problems().unwrap();
    println!("problems: {}", problems.len());
    for p in problems.iter().take(20) {
        println!("  {} {:?}", p.path, p.problem);
    }
}
