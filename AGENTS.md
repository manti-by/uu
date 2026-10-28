# AGENTS.md

Guidance for agents and contributors working on `uu` itself.

## What this project is

`uu` is a CLI that bumps the dependency minimum versions in `pyproject.toml` to the versions the project's `uv.lock` resolved to. It is a Rust port of
[`uv-bump`](https://github.com/zundertj/uv-bump), preserving that tool's observable behaviour: which specifier operators are rewritten, which are left
alone, the exact wording of the verbose table, and the fact that `pyproject.toml` files keep their comments, spacing and unrelated tables untouched. Rust, edition 2024.

## Commands

| Command | Purpose |
| --- | --- |
| `make build` | Build the binary into `target/debug/uu`. |
| `make run` | Run all tests. Unit, integration and offline smoke; none touch the network beyond a real `uv sync` in the manual demo. |
| `make check` | `fmt --check`, clippy and tests; the gate before committing. |
| `make release` | Runs the `make check` gate, then builds `target/release/uu`. |
| `make install` | `cargo install --path .`, into `CARGO_HOME/bin`. What `wiki/docs/INSTALL.md` tells users. |
| `make clean` | `cargo clean`. |
| `cargo build` | The same build, without the Makefile. |
| `cargo test` | The same tests, without the Makefile. |
| `cargo clippy --all-targets` | Lint. Must stay warning-free. |
| `cargo fmt` | Format. `cargo fmt --check` must be clean. |

## Layout

| Path | Role |
| --- | --- |
| `src/main.rs` | Argument parsing and dispatch. Keeps no logic beyond wiring. |
| `src/bump.rs` | The core `bump` pipeline: snapshots the lock, syncs, rewrites each `pyproject.toml`, prints the verbose table. |
| `src/pyproject.rs` | The regex rewrite of a `pyproject.toml` text — every semantic rule about which specifiers move lives here. |
| `src/lock.rs` | `uv.lock` parsing: package versions in lock order, and where every workspace member's `pyproject.toml` sits. |
| `src/uv.rs` | The `uv sync --all-extras [--upgrade]` invocation. The only place that knows how uv is called. |
| `src/progress.rs` | The `Checking updates...` indicator with its in-place three-dot animation, plus the decision of when dots may animate. |
| `src/error.rs` | The `Error` type, `Result`, and the `bail!`/`ensure!` macros. |
| `src/lib.rs` | Module tree; the testable surface the integration tests import. |
| `tests/e2e.rs` | End-to-end tests mirroring the upstream `uv-bump` test suite, with a fake resolver writing the lock. |
| `tests/resources/` | Fixtures, including a trimmed `uv_workspaces.lock`. |
| `README.md` | The front door: what `uu` is, install, links out. Kept short on purpose. |
| `wiki/docs/` | User-facing docs. `INSTALL.md` covers install options; `MORE.md` owns the command list and pyproject semantics reference. |

## Conventions

- **Behaviour parity beats elegance.** The upstream tool edits `pyproject.toml` with one regex per package over the raw text; `src/pyproject.rs` does
  exactly that with the `regex` crate, keeping the same group numbering, the same replacement shape and the same line-diff counting. Reparsing the TOML
  (for example with `toml_edit`) would change formatting behaviour and is a bug even when tests still pass.
- Dependencies stay minimal on purpose: `regex`, `serde`, `toml`. No async runtime, no HTTP client — the only subprocess is `uv sync`.
- Errors are `Error::new` for anything the user can fix (a missing `uv.lock`, a blocked install directory) and `Error::with_source` when wrapping an
  underlying I/O or process failure, so the printed chain reads `could not read pyproject.toml: No such file…`. `main` prints the whole chain.
- Every module that can be tested in isolation carries a `#[cfg(test)] mod tests` block. The full-pipeline tests live in `tests/e2e.rs` and inject a fake
  resolver closure, so they never call the real `uv`.
- Progress and status go to stderr, tables and other user-offered detail to
  stdout, so piped stdout stays free of decoration. The dot animation rewrites
  one line with `\r` rather than appending, and only
  runs when stderr is a real terminal (`std::io::IsTerminal`); piped output
  sees plain lines. Stdout and stderr must be passed to `bump` as the streams
  themselves, never as a held `lock()` — the spinner thread locks stderr
  while the main thread writes through it, and a lock held across the run is
  a deadlock.
- The verbose table is part of the output contract. The literal tab counts and the `→` separator match upstream; do not restyle it without checking the
  upstream output string.
- Anything the user sees is documented in exactly one place. `README.md` is the short front door and links out; `wiki/docs/MORE.md` owns the command list
  and the specifier-semantics reference; `wiki/docs/INSTALL.md` owns install. Do not restate a command list in more than one file, and keep console
  transcripts copied from a real run of the binary rather than from memory. `wiki/README.md` documents these rules under "Docs".

## Working on this repository

`sk.toml`, `sk.lock` and `.agents/skills/` are this project's own manifest, lock and installed skills. Run `sk sync` after editing `sk.toml`, and commit the
refreshed lock with the manifest change — each entry carries the SHA-256 of the installed `SKILL.md`, so the lock diff shows whether content actually moved.

Releases are cut by CI from a version bump, not by hand: bump `version` in `Cargo.toml`, merge to `master`, and `.github/workflows/release.yml` builds the
four platform binaries and publishes `v<version>`. The publish job is gated behind `make check`, so a red tree never releases. The release job is
idempotent: a manual `workflow_dispatch` re-runs the publish and `--clobber`s existing assets, which is how a half-finished release is repaired.
`.github/workflows/ci.yml` runs that gate plus an offline smoke test of `install.sh` on every push and pull request.

The two manifests carry unrelated versions; they are not connected and must not be synchronised. `Cargo.toml`'s `version` is the application's: it is what
`uu --version` prints and the only version `release.yml` reads when deciding whether to cut `v<version>`. `sk.toml`'s `project.version` versions this
repository's agent-skills manifest and nothing else — no binary, tag, release or install asset depends on it. A gap between the two is expected state, not
drift, so do not bump one to match the other; `sk.lock` is what tracks skill content, pinning each `SKILL.md` by commit and SHA-256.

Both workflows keep their shell in `.github/scripts/`, one script per step, rather than inlined `run:` blocks. An inline script gets no syntax
highlighting, no editor tooling and no shellcheck; a file in that directory gets all three, and `ci.yml` lints the whole directory on every push. The
scripts are `bash`, take their inputs from the environment rather than arguments so the workflow can pass them through `env:` instead of interpolating
`${{ … }}` into shell text, and assume they run from the repository root. `install.sh` alone is POSIX `sh`, because `curl … | sh` must work where macOS
ships bash 3.2 and Ubuntu ships dash.
