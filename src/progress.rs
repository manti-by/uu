//! Progress reporting: the "Checking updates..." line with a dot animation,
//! written to stderr so stdout stays free of decoration.
//!
//! The dots only animate when stderr is a real terminal. Piped output — CI
//! logs, `uu -v | tee` — sees plain lines ending in a newline, which is also
//! what makes the module deterministic under test.

use std::io::{IsTerminal, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const CHECKING_MESSAGE: &str = "Checking updates...";
const DOT_INTERVAL: Duration = Duration::from_millis(250);

/// Whether stderr is a terminal, and therefore whether the dots may animate.
pub fn stderr_is_terminal() -> bool {
    std::io::stderr().is_terminal()
}

/// A running "Checking updates..." indicator. The line starts without a
/// trailing newline; dots append to it while the work runs, and `finish`
/// terminates the animation and ends the line.
pub struct Checking {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    #[allow(dead_code)] // kept for symmetry and for future finish variants
    animated: bool,
}

impl Checking {
    /// Stops the dots and terminates the line with a newline.
    pub fn finish(mut self, progress: &mut dyn Write) -> std::io::Result<()> {
        if let Some(handle) = self.handle.take() {
            self.stop.store(true, Ordering::SeqCst);
            let _ = handle.join();
        }
        progress.write_all(b"\n")?;
        progress.flush()
    }
}

impl Drop for Checking {
    fn drop(&mut self) {
        // A panicked or early-returned `bump` must not leave the dot thread
        // spinning behind the error message; joining here is safe either way.
        if let Some(handle) = self.handle.take() {
            self.stop.store(true, Ordering::SeqCst);
            let _ = handle.join();
        }
    }
}

/// Prints the checking line and, when `animated`, keeps appending a dot every
/// interval until `finish`. The animation writes to stderr directly: the dots
/// belong to whatever the caller passed as the progress sink there.
pub fn start(progress: &mut dyn Write, animated: bool) -> std::io::Result<Checking> {
    progress.write_all(CHECKING_MESSAGE.as_bytes())?;
    progress.flush()?;

    if !animated {
        return Ok(Checking {
            stop: Arc::new(AtomicBool::new(true)),
            handle: None,
            animated,
        });
    }

    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);
    let handle = thread::spawn(move || {
        while !flag.load(Ordering::Relaxed) {
            thread::sleep(DOT_INTERVAL);
            if flag.load(Ordering::Relaxed) {
                break;
            }
            let mut stderr = std::io::stderr().lock();
            let _ = stderr.write_all(b".");
            let _ = stderr.flush();
        }
    });

    Ok(Checking {
        stop,
        handle: Some(handle),
        animated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_output_is_a_single_line() {
        let mut progress: Vec<u8> = Vec::new();
        let checking = start(&mut progress, false).unwrap();
        checking.finish(&mut progress).unwrap();
        assert_eq!(
            String::from_utf8(progress).unwrap(),
            "Checking updates...\n"
        );
    }

    #[test]
    fn finishing_stops_the_animation() {
        let mut progress: Vec<u8> = Vec::new();
        let checking = start(&mut progress, true).unwrap();
        checking.finish(&mut progress).unwrap();
        // The prefix plus the terminator land in the sink; dots go to stderr.
        let text = String::from_utf8(progress).unwrap();
        assert!(text.starts_with("Checking updates..."));
        assert!(text.ends_with('\n'));
    }
}
