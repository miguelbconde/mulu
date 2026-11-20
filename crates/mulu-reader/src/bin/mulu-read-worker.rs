//! A standalone reading process, used by the tests. Mulu's own program runs the same
//! code when started with `--read-worker`.

#[global_allocator]
static ALLOCATOR: mulu_reader::MemoryCap = mulu_reader::memory_cap();

fn main() {
    let settings = mulu_reader::Settings::from_args(std::env::args().skip(1));
    mulu_reader::serve(&ALLOCATOR, settings);
}
