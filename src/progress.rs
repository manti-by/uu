//! Progress reporting: the "Checking updates..." line with a three-dot
//! animation, written to stderr so stdout stays free of decoration.
//!
//! While the work runs the dots cycle in place — a carriage return, then the
//! same line again with one more dot, padded to a fixed width so the frame
//! never jitters. The dots only animate when stderr is a real terminal. Piped
//! output — CI logs, `uu -v | tee` — sees one plain `Checking updates...` line
//! and no carriage returns, which is also what makes the module deterministic
//! under test.

use std::io::{IsTerminal, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const CHECKING_MESSAGE: &str = "Checking updates";
const DOT_COUNT: usize = 3;
const DOT_INTERVAL: Duration = Duration::from_millis(250);

/// The resolved line: what piped output prints, and what the animation
/// collapses to once the work is done.
fn checking_line() -> String {
    format!("{CHECKING_MESSAGE}{}", ".".repeat(DOT_COUNT))
}

/// The animated line's width, dots and their padding included. Every frame is
/// padded to exactly this many columns, so rewriting the line in place never
/// leaves a stale dot behind — and the same width clears it at the end.
const LINE_WIDTH: usize = CHECKING_MESSAGE.len() + DOT_COUNT + 1;

/// One animation frame: `dots` dots, padded to `LINE_WIDTH`. The animation
/// runs 1 → 3 dots and wraps, so the line is never dot-less.
fn frame(dots: usize) -> String {
    format!(
        "{CHECKING_MESSAGE}{:padding$}",
        ".".repeat(dots),
        padding = DOT_COUNT + 1
    )
}

/// Whether stderr is a terminal, and therefore whether the dots may animate.
pub fn stderr_is_terminal() -> bool {
    std::io::stderr().is_terminal()
}

/// A running "Checking updates..." indicator. The line starts without a
/// trailing newline; frames rewrite it while the work runs, and `finish`
/// terminates the animation and ends the line.
pub struct Checking {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    animated: bool,
}

impl Checking {
    /// Stops the dots, clears the animated frame and terminates the line with
    /// a newline. A piped run just ends the plain line it already wrote.
    pub fn finish(mut self, progress: &mut dyn Write) -> std::io::Result<()> {
        if let Some(handle) = self.handle.take() {
            self.stop.store(true, Ordering::SeqCst);
            let _ = handle.join();
        }

        if self.animated {
            // Wipe the frame, then resolve to the finished line once, so the
            // scrollback reads `Checking updates...` rather than a frozen
            // intermediate frame.
            write!(
                progress,
                "\r{blank}\r{line}\n",
                blank = " ".repeat(LINE_WIDTH),
                line = checking_line()
            )?;
            return progress.flush();
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

/// Prints the checking line and, when `animated`, keeps rewriting it with the
/// next frame until `finish`. The animation writes to stderr directly: the
/// dots belong to whatever the caller passed as the progress sink there.
pub fn start(progress: &mut dyn Write, animated: bool) -> std::io::Result<Checking> {
    if !animated {
        progress.write_all(checking_line().as_bytes())?;
        progress.flush()?;
        return Ok(Checking {
            stop: Arc::new(AtomicBool::new(true)),
            handle: None,
            animated: false,
        });
    }

    // The first frame lands synchronously, so the line is on screen before
    // the resolver is asked to do anything.
    progress.write_all(frame(1).as_bytes())?;
    progress.flush()?;

    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);
    let handle = thread::spawn(move || {
        let mut dots = 1;
        while !flag.load(Ordering::Relaxed) {
            thread::sleep(DOT_INTERVAL);
            if flag.load(Ordering::Relaxed) {
                break;
            }
            dots = dots % DOT_COUNT + 1;
            let mut stderr = std::io::stderr().lock();
            let _ = write!(stderr, "\r{}", frame(dots));
            let _ = stderr.flush();
        }
    });

    Ok(Checking {
        stop,
        handle: Some(handle),
        animated: true,
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
    fn frames_cycle_one_to_three_dots_without_changing_width() {
        let frames: Vec<String> = (1..=DOT_COUNT).map(frame).collect();
        assert_eq!(
            frames,
            [
                "Checking updates.   ",
                "Checking updates..  ",
                "Checking updates... ",
            ]
        );
        for animated_frame in &frames {
            assert_eq!(animated_frame.len(), LINE_WIDTH);
        }
    }

    #[test]
    fn finishing_clears_the_animated_frame() {
        let mut progress: Vec<u8> = Vec::new();
        let checking = start(&mut progress, true).unwrap();
        checking.finish(&mut progress).unwrap();
        // The first frame plus the wipe land in the sink; later frames go to
        // stderr. Nothing but the resolved line survives the wipe.
        let text = String::from_utf8(progress).unwrap();
        assert_eq!(
            text,
            format!(
                "{}\r{}\rChecking updates...\n",
                frame(1),
                " ".repeat(LINE_WIDTH)
            )
        );
    }
}
