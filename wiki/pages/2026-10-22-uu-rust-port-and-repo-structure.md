# uu: the Rust port of uv-bump

2026-10-22 — why `uu` exists and why it is a port rather than a new idea.

## Context

[uv-bump](https://github.com/zundertj/uv-bump) solves one real problem: when
an application's `uv.lock` is upgraded, the `pyproject.toml` minimum bounds
keep lagging behind what the code actually depends on. The tool is Python
with a uv dependency; a fresh-project invocation pays for an interpreter and
import machinery where the actual work is a regex rewrite over a text file.

## Decision

Port `uv-bump` to Rust as `uu`, keeping observable behaviour identical to the
upstream tool, and structure the repository after `manti-by/sk`:

- The pyproject rewrite keeps upstream's single-regex-per-package approach on
  the raw text, preserving group numbering and the replacement shape, so
  comments, spacing and unrelated tables survive exactly as in the original.
- The verbose table matches upstream's output string, tabs included.
- `uv sync` stays the resolver, shelled out to as a subprocess.
- The repository reuses the `sk` CI layout: `.github/scripts/` with one
  script per step, `ci.yml` running the `make check` gate plus an offline
  `install.sh` smoke test, `release.yml` building four platform binaries on
  version bumps and publishing them as `v<version>` GitHub releases, and a
  POSIX-sh `install.sh` that verifies checksums before anything is installed.

## Consequences

- Feature parity is the test suite. The per-specifier cases — upper bounds
  kept, extras preserved, `==` pins untouched, comments kept — are unit tests
  in `src/pyproject.rs`; `tests/e2e.rs` mirrors the upstream suite's pipeline
  cases, both verbose columns included.
- First version is `0.1.0`, released by pushing the version bump to `master`.
- Any behaviour difference that a port intentionally makes (for example a
  clearer error message naming the missing file) is fine as long as the data
  written and the exit semantics stay the same; differences must be checked
  in, not accidental.

Amended 2026-10-28: the parity bullet above credited `tests/e2e.rs` with the
per-specifier cases. They actually live in `src/pyproject.rs`'s unit tests
(`updates_upper_bound_kept_intact`, `updates_extras`,
`leaves_exact_pins_alone`, `keeps_inline_comment`, …); the end-to-end suite
covers the pipeline and both verbose columns. Wording corrected during a wiki
consistency pass, decision unchanged.
