# Install

The one-liner, on macOS and Linux:

```
curl -LsSf https://raw.githubusercontent.com/manti-by/uu/master/install.sh | sh
```

The script downloads the newest release for your machine, verifies its
SHA-256 against the release's published `.sha256` file and installs the
binary into `~/.local/bin`. Requires `curl` or `wget`, `tar` and any one of
`sha256sum`, `shasum` or `openssl`.

## Options and environment

`install.sh --help` prints the full list. The short version:

| Flag / variable | Meaning | Default |
| --- | --- | --- |
| `--dir <path>` / `UU_INSTALL_DIR` | where the binary goes | `~/.local/bin` |
| `--version <tag>` / `UU_VERSION` | install a specific release instead of the newest one | newest release |
| `UU_REPO` | another `owner/repo` to pull releases from | `manti-by/uu` |
| `UU_RELEASE_BASE` | override the release download base URL (used by the CI smoke test) | GitHub releases |
| `GITHUB_TOKEN` / `GH_TOKEN` | token for the GitHub API, raising the anonymous rate limit | unset |

Examples:

```
# a self-managed directory
curl -LsSf https://raw.githubusercontent.com/manti-by/uu/master/install.sh | UU_INSTALL_DIR=/opt/bin sh

# pin a version
curl -LsSf https://raw.githubusercontent.com/manti-by/uu/master/install.sh | sh -s -- --version v0.1.0
```

If `~/.local/bin` is not on your `PATH`, the installer says so when it ends;
add it before running `uu`.

## With cargo

`uu` builds with a stable toolchain and no system dependencies:

```
cargo install --git https://github.com/manti-by/uu
```

or, from a checkout, `make install`.

## Manual

Every release ships one `tar.gz` per platform plus a `checksums.txt`
aggregate:

| Asset | Platform |
| --- | --- |
| `uu-<version>-linux-x86_64.tar.gz` | Linux on x86_64 |
| `uu-<version>-linux-aarch64.tar.gz` | Linux on arm64 |
| `uu-<version>-macos-x86_64.tar.gz` | macOS on Intel |
| `uu-<version>-macos-aarch64.tar.gz` | macOS on Apple Silicon |

Each archive is a `tar.gz` holding the `uu` binary at its root, next to
`README.md` and `LICENSE`. Download, verify against `checksums.txt` or
`<archive>.sha256`, extract and put the binary wherever your `PATH` finds it.
