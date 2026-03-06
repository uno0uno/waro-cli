use colored::Colorize;
use std::io::{IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// 1x4 matrix — one cell lit at a time (pure ASCII)
const FRAMES: [&str; 4] = ["[#...]", "[.#..]", "[..#.]", "[...#]"];

const MESSAGES: [&str; 8] = [
    "Fetching your orders...",
    "Loading the menu...",
    "Preparing your data...",
    "Checking today's sales...",
    "Serving your data...",
    "Reading the menu board...",
    "Plating the response...",
    "Almost ready to serve...",
];

/// Print the CLI header. Only renders when stderr is a TTY.
pub fn print_welcome() {
    if !std::io::stderr().is_terminal() {
        return;
    }
    eprintln!("{}", "waro-cli".bold());
    eprintln!("{}", "Welcome to WaRo Colombia".dimmed());
    eprintln!();
}

/// Animated spinner that runs in a background thread.
/// Output goes to stderr so stdout (JSON data) stays clean.
pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    /// Start the spinner. Returns immediately; animation runs in background.
    /// No-op when stderr is not a TTY (e.g. piped output).
    pub fn start() -> Self {
        let running = Arc::new(AtomicBool::new(true));

        let handle = if std::io::stderr().is_terminal() {
            let running_clone = running.clone();
            Some(thread::spawn(move || {
                let mut frame_i: usize = 0;
                let mut msg_i: usize = 0;
                loop {
                    if !running_clone.load(Ordering::Relaxed) {
                        break;
                    }
                    let frame = FRAMES[frame_i % 4];
                    let msg = MESSAGES[msg_i % MESSAGES.len()];
                    eprint!("\r  {}  {}", frame, msg);
                    let _ = std::io::stderr().flush();
                    thread::sleep(Duration::from_millis(150));
                    frame_i += 1;
                    // Advance message every 8 frames (~1.2 s)
                    if frame_i % 8 == 0 {
                        msg_i += 1;
                    }
                }
                // Clear the spinner line before returning control
                eprint!("\r{:<60}\r", "");
                let _ = std::io::stderr().flush();
            }))
        } else {
            None
        };

        Spinner { running, handle }
    }

    /// Stop the spinner and clear the line.
    pub fn stop(self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(h) = self.handle {
            let _ = h.join();
        }
    }
}
