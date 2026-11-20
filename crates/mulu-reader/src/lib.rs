//! # mulu-reader
//!
//! Files from a library are untrusted (project principles): a damaged or malicious file
//! could make a reader loop forever, use all the memory, or crash. So Mulu never reads
//! a file in its own process. A separate **reading process** reads one file at a time
//! and sends back its text; Mulu stays responsive whatever happens there (decision D25).
//!
//! Protections:
//! - **Time:** if a file takes longer than the time limit, the reading process is
//!   stopped and a new one is started for the next file. The reading process also stops
//!   itself if it is still busy well after the limit, so that it never outlives Mulu.
//! - **Memory:** every allocation of the reading process is counted; past its allowance
//!   the allocation is refused and the process ends, reported as "too large". This works
//!   the same on Windows, macOS and Linux, unlike the systems' own limits (macOS does
//!   not enforce memory limits on a process).
//! - **Crash:** if the reading process stops for any other reason, the file is reported
//!   and a new process reads the next one.
//! - **Priority:** the reading process runs at low priority, so the computer stays
//!   responsive while a library is indexed.
//! - The reading process only reads: it receives a file path and returns text.
//!
//! The reading process is Mulu's own program started with [`WORKER_FLAG`]; it calls
//! [`serve`]. Its `main` must install [`MemoryCap`] as the global allocator.

use mulu_extract::{Document, Limits, Problem};
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// The command-line flag that starts Mulu's program as a reading process.
pub const WORKER_FLAG: &str = "--read-worker";

/// Default memory allowance of the reading process.
pub const MEMORY_LIMIT: usize = 1024 * 1024 * 1024;

/// Default time limit for one file.
pub const TIME_LIMIT: Duration = Duration::from_secs(120);

/// The allocator that enforces the memory allowance; install it in the program's `main`
/// file: `#[global_allocator] static ALLOCATOR: MemoryCap = mulu_reader::memory_cap();`
/// The allowance only takes effect in the reading process (see [`serve`]).
pub type MemoryCap = cap::Cap<std::alloc::System>;

/// A [`MemoryCap`] with no limit yet.
pub const fn memory_cap() -> MemoryCap {
    cap::Cap::new(std::alloc::System, usize::MAX)
}

#[derive(Serialize, Deserialize)]
struct Request {
    path: PathBuf,
}

#[derive(Serialize, Deserialize)]
enum Response {
    Document(Document),
    Problem(Problem),
}

/// Settings of a reading process, passed to it on its command line.
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub memory_limit: usize,
    pub time_limit: Duration,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            memory_limit: MEMORY_LIMIT,
            time_limit: TIME_LIMIT,
        }
    }
}

impl Settings {
    fn to_args(self) -> Vec<OsString> {
        vec![
            WORKER_FLAG.into(),
            format!("--memory={}", self.memory_limit).into(),
            format!("--seconds={}", self.time_limit.as_secs_f64()).into(),
        ]
    }

    /// Reads the settings from the reading process's command line.
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Settings {
        let mut settings = Settings::default();
        for arg in args {
            if let Some(n) = arg.strip_prefix("--memory=").and_then(|v| v.parse().ok()) {
                settings.memory_limit = n;
            }
            if let Some(s) = arg
                .strip_prefix("--seconds=")
                .and_then(|v| v.parse::<f64>().ok())
            {
                settings.time_limit = Duration::from_secs_f64(s.max(0.0));
            }
        }
        settings
    }
}

/// Runs the reading process: reads one request per line from standard input and
/// answers one line per request on standard output, until standard input closes
/// (which happens when Mulu closes).
pub fn serve(allocator: &MemoryCap, settings: Settings) {
    // Started at most a few milliseconds after the program, before any file is read.
    let _ = allocator.set_limit(allocator.allocated().saturating_add(settings.memory_limit));

    // Watchdog: a file still being read long after the limit means Mulu is gone or
    // could not stop this process; stop it here.
    let busy_since = Arc::new(AtomicU64::new(0));
    let start = Instant::now();
    {
        let busy_since = Arc::clone(&busy_since);
        let grace = settings.time_limit + Duration::from_secs(10);
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(500));
                let since = busy_since.load(Ordering::Relaxed);
                if since != 0
                    && start.elapsed().saturating_sub(Duration::from_millis(since)) > grace
                {
                    std::process::exit(70);
                }
            }
        });
    }

    let limits = Limits::default();
    let stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lines() {
        let Ok(line) = line else { break };
        let response = match serde_json::from_str::<Request>(&line) {
            Ok(request) => {
                let now = u64::try_from(start.elapsed().as_millis())
                    .unwrap_or(u64::MAX)
                    .max(1);
                busy_since.store(now, Ordering::Relaxed);
                let result = mulu_extract::extract(&request.path, &limits);
                busy_since.store(0, Ordering::Relaxed);
                match result {
                    Ok(document) => Response::Document(document),
                    Err(problem) => Response::Problem(problem),
                }
            }
            Err(e) => Response::Problem(Problem::Unreadable(format!("bad request: {e}"))),
        };
        let Ok(text) = serde_json::to_string(&response) else {
            break;
        };
        if writeln!(stdout, "{text}")
            .and_then(|()| stdout.flush())
            .is_err()
        {
            break;
        }
    }
}

/// Mulu's side: sends files to a reading process and enforces the time limit.
pub struct Reader {
    program: PathBuf,
    settings: Settings,
    worker: Option<Worker>,
}

struct Worker {
    child: Child,
    stdin: ChildStdin,
    answers: Receiver<String>,
    errors: Arc<Mutex<String>>,
}

impl Reader {
    /// A reader that starts `program` (normally Mulu's own program, from
    /// `std::env::current_exe()`) as its reading process when first needed.
    pub fn new(program: impl Into<PathBuf>, settings: Settings) -> Reader {
        Reader {
            program: program.into(),
            settings,
            worker: None,
        }
    }

    /// Reads one file in the reading process.
    pub fn read(&mut self, path: &Path) -> Result<Document, Problem> {
        let request = serde_json::to_string(&Request {
            path: path.to_owned(),
        })
        .map_err(|e| Problem::Unreadable(e.to_string()))?;
        // A process that ended while idle is replaced once, without blame on this file.
        for attempt in 0..2 {
            let worker = self.worker()?;
            if writeln!(worker.stdin, "{request}")
                .and_then(|()| worker.stdin.flush())
                .is_ok()
            {
                break;
            }
            self.stop();
            if attempt == 1 {
                return Err(Problem::ReaderStopped);
            }
        }
        let time_limit = self.settings.time_limit;
        let worker = self.worker.as_mut().ok_or(Problem::ReaderStopped)?;
        match worker.answers.recv_timeout(time_limit) {
            Ok(line) => match serde_json::from_str::<Response>(&line) {
                Ok(Response::Document(document)) => Ok(document),
                Ok(Response::Problem(problem)) => Err(problem),
                Err(_) => {
                    self.stop();
                    Err(Problem::ReaderStopped)
                }
            },
            Err(RecvTimeoutError::Timeout) => {
                self.stop();
                Err(Problem::TimedOut)
            }
            Err(RecvTimeoutError::Disconnected) => {
                let out_of_memory = self.stop_and_report().contains("memory allocation");
                Err(if out_of_memory {
                    Problem::TooLarge
                } else {
                    Problem::ReaderStopped
                })
            }
        }
    }

    /// The system's identifier of the current reading process, if one is running.
    pub fn process_id(&self) -> Option<u32> {
        self.worker.as_ref().map(|w| w.child.id())
    }

    fn worker(&mut self) -> Result<&mut Worker, Problem> {
        if self.worker.is_none() {
            self.worker = Some(
                self.start()
                    .map_err(|e| Problem::Unreadable(e.to_string()))?,
            );
        }
        self.worker.as_mut().ok_or(Problem::ReaderStopped)
    }

    fn start(&self) -> std::io::Result<Worker> {
        let mut command = low_priority(&self.program);
        command
            .args(self.settings.to_args())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| std::io::Error::other("no stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| std::io::Error::other("no stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| std::io::Error::other("no stderr"))?;
        let (sender, answers) = channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        let errors = Arc::new(Mutex::new(String::new()));
        {
            let errors = Arc::clone(&errors);
            thread::spawn(move || {
                let mut buffer = [0u8; 4096];
                let mut stderr = stderr;
                while let Ok(n) = stderr.read(&mut buffer) {
                    if n == 0 {
                        break;
                    }
                    if let Ok(mut tail) = errors.lock() {
                        tail.push_str(&String::from_utf8_lossy(&buffer[..n]));
                        if tail.len() > 4096 {
                            let cut = tail.len() - 2048;
                            let cut = (cut..tail.len())
                                .find(|&i| tail.is_char_boundary(i))
                                .unwrap_or(0);
                            tail.drain(..cut);
                        }
                    }
                }
            });
        }
        Ok(Worker {
            child,
            stdin,
            answers,
            errors,
        })
    }

    fn stop(&mut self) {
        self.stop_and_report();
    }

    /// Stops the reading process and returns the end of what it wrote on its error
    /// output.
    fn stop_and_report(&mut self) -> String {
        let Some(mut worker) = self.worker.take() else {
            return String::new();
        };
        let _ = worker.child.kill();
        let _ = worker.child.wait();
        // Give the error-output thread a moment to collect the last lines.
        thread::sleep(Duration::from_millis(50));
        worker.errors.lock().map(|t| t.clone()).unwrap_or_default()
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        self.stop();
    }
}

/// The command for the reading process, at low priority so the computer stays
/// responsive during indexing.
fn low_priority(program: &Path) -> Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x0000_4000;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut command = Command::new(program);
        command.creation_flags(BELOW_NORMAL_PRIORITY_CLASS | CREATE_NO_WINDOW);
        command
    }
    #[cfg(unix)]
    {
        // `nice` is part of every macOS and Linux system.
        if Path::new("/usr/bin/nice").exists() {
            let mut command = Command::new("/usr/bin/nice");
            command.arg("-n").arg("10").arg(program);
            command
        } else {
            Command::new(program)
        }
    }
    #[cfg(not(any(windows, unix)))]
    {
        Command::new(program)
    }
}
