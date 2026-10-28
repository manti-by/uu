# Three dots animate in place

2026-10-28 — replacing the appended dot trail with a 1 → 3 dot cycle that
rewrites one line.

## Context

The indicator from [[2026-10-22-progress-output-and-tty-animation]] appended a
dot every 250 ms. That reads as progress by line length: a slow resolve keeps
growing the line, wraps it on a narrow terminal, and leaves a stale trail in
scrollback that says nothing once the run is over. The user asked for "just 3
animated dots" instead, which means rewriting the line rather than extending
it.

## Decision

The checking line animates in place. Each frame is the message, one to three
dots, and padding to a fixed 20 columns, written as `\r{frame}` every 250 ms
and wrapped 3 → 1 so the line is never dot-less. The first frame is written
synchronously before `uv sync` starts, so the line exists even if the resolver
is instantaneous.

`finish` joins the animation thread, wipes the frame with `\r` + 20 spaces +
`\r`, then writes `Checking updates...` once. Piped output is unchanged: when
stderr is not a terminal nothing animates and the run writes exactly one
`Checking updates...` line, with no carriage returns.

## Consequences

- The fixed width means no stale dots, no wrapping, and a single resolved line
  in scrollback — a frozen intermediate frame never survives.
- Clearing happens after the join, inside `finish`, so a frame cannot land
  after the wipe; the same ordering keeps a failing `uv sync` from leaving a
  live animation behind its error.
- The piped contract and its test (`plain_output_is_a_single_line`) did not
  move. The animation tests assert the three frame strings and the exact
  wipe-plus-resolve output.
- The 250 ms interval, the 1 → 3 cycle and the width all live in
  `src/progress.rs` next to the animation, so tuning cannot drift from the
  codes that render it.
