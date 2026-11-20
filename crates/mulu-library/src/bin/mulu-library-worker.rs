//! The reading process for this crate's tests (Mulu's program does the same with
//! `--read-worker`).

#[global_allocator]
static ALLOCATOR: mulu_reader::MemoryCap = mulu_reader::memory_cap();

fn main() {
    let settings = mulu_reader::Settings::from_args(std::env::args().skip(1));
    mulu_reader::serve(&ALLOCATOR, settings);
}
