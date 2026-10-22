# UU

`uu` keeps your `pyproject.toml` dependency minimum versions in sync with what your `uv.lock` file actually resolved to.

[![CI](https://github.com/manti-by/uu/actions/workflows/ci.yml/badge.svg)](https://github.com/manti-by/uu/actions/workflows/ci.yml)

`uv sync --upgrade` updates `uv.lock`, but never touches `pyproject.toml` — so over time a manifest that says `"six>=1.16.0"` starts depending on features of 1.17.0. `uu` runs `uv sync` for you and then bumps every feasible minimum bound in the manifest to the version the lock resolved:

```
$ uu -v
Processed pyproject.toml
	Package		Lock file		pyproject.toml
	python-dateutil		-		2.0 → 3.0.0
	six		-		1.16.0 → 1.17.0
```

Use `--no-upgrade`/`-n` to skip re-resolving the lock and only rewrite the manifest against the versions already in it, and `--pyproject-toml <file>` to point at a specific root file (workspaces are handled: every member's `pyproject.toml` is bumped too).

## Installation

The quickest way, on macOS and Ubuntu, is the install script. It downloads the newest release for your machine, verifies its SHA-256 and drops the binary into `~/.local/bin`:

```
$ curl -LsSf https://raw.githubusercontent.com/manti-by/uu/master/install.sh | sh
```
For more installation options check [INSTALL.md](wiki/docs/INSTALL.md).

## Commands

Every command and its options are listed in [MORE.md](wiki/docs/MORE.md), which also covers how version boundaries work (`==`/`<=` are never touched, `>=`/`~=`/`>` are bumped) and where the idea of `uu` originates (upstream: [uv-bump](https://github.com/zundertj/uv-bump), a Python tool; `uu` is its Rust port — faster startup, no Python needed).

## Development

`make check` runs formatting, clippy and the tests. Releases are cut by CI: bump `version` in `Cargo.toml`, merge to `master`, and four platform binaries are published as `v<version>`. Everything worth knowing about the layout lives in [AGENTS.md](AGENTS.md).
