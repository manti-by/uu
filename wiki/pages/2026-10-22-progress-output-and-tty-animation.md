# Progress output lives on stderr, animated only on a tty

2026-10-22 — how `uu` reports what it is doing without breaking piped output,
and the deadlock that shaped the implementation.

## Context

The default run used to print nothing: `uv sync` output was swallowed, the
table came only with `-v`, and a user staring at a slow resolve saw nothing.
Adding status lines risked polluting stdout, which is the stream worth
keeping clean for scripts.

## Decision

- Every run prints `Checking updates...` with a dot-per-interval animation,
  then either one line per updated package (`  name old → new`) or
  `Everything up to date.`
- All of that goes to stderr; stdout keeps only the `-v` tables.
- The animation runs only when stderr is a real terminal
  (`std::io::IsTerminal`). Piped output — including every captured transcript
  in the docs and in tests — sees the plain line, which is what makes the
  module deterministic under test.
- The error path always finishes the indicator before the error is reported,
  so a failing `uv sync` cannot leave a live animation behind the error.

## Consequences

- Streams must be passed to `bump` as `Stdout`/`Stderr` themselves, NOT as a
  held `stdout.lock()`/`stderr.lock()`: the animation thread locks stderr
  while the main thread writes through the same handle, and a lock held
  across the whole run deadlocks the tool exactly when stderr is a real
  terminal — which is when animation runs. The first tty test exposed this.
- The dot interval (250 ms) and message wording live in `src/progress.rs`
  next to the animation; output semantics are documented in
  `wiki/docs/MORE.md`.
- Future upgrades (per-package progress during sync, ANSI line clearing)
  would most naturally come from a dedicated progress crate, at the cost of
  dropping the std-only dependency rule; the current shape was kept std-only
  in the spirit of `sk`'s `parallel.rs`.

The animation's shape was revised on 2026-10-28: the appended dot trail became
a fixed-width 1 → 3 dot cycle on the same line, as recorded in
[[2026-10-28-in-place-three-dot-animation]]. Everything above — stderr,
tty-only animation, finishing before the error — still holds.
