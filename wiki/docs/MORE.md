# More

Everything the README leaves out: the full command list, the pyproject
semantics, workspaces and how `uv sync` is involved.

## Running

`uu` takes options only; it is run inside your project directory and works on
that project.

```
$ uu --help
uu - updates your pyproject.toml minimum versions to the latest feasible version

USAGE:
    uu [OPTIONS]

OPTIONS:
    --pyproject-toml <file>  File to update; for a workspace pass the root one
    -v, --verbose            Report the changes made per pyproject.toml file
    -n, --no-upgrade         Don't upgrade dependencies as part of `uv sync`
    -h, --help               Print this help
    -V, --version            Print the version

WARNING: this tool is provided as-is and doesn't come with warranty. Please
make sure your code has been backed up and/or version controlled in case
something goes wrong.
```

## How a bump works

1. The package versions are read out of `uv.lock` — the *before* snapshot.
2. `uv sync --all-extras` is run (`--upgrade` unless `-n` was passed). uv
   resolves and writes back the lock; `uu` swallows uv's output exactly like
   the upstream tool does, so a failing sync surfaces only its exit code and
   stderr through `uu`'s error.
3. The lock is re-read — the *after* snapshot.
4. Every `pyproject.toml` the project owns (all workspace members, see below)
   is rewritten: each feasible `>=`, `~=` or `>` minimum bound becomes an
   exact `>=` bound at the version the lock resolved.

All of step 4 operates on the raw file text with one regular expression per
package, so comments, spacing and unrelated tables survive. A file is written
back even when nothing changed, matching upstream behaviour.

## What moves and what never moves

`uu` respects the bounds you already set:

| Declaration | After a bump to 1.21.0 | Why |
| --- | --- | --- |
| `"polars>=1.20.0"` | `"polars>=1.21.0"` | the minimum bound is moved |
| `"polars>=1.20.0,<1.22"` | `"polars>=1.21.0,<1.22"` | the upper bound is kept; 1.21 is feasible |
| `"polars~=1.20.0"` | `"polars>=1.21.0"` | any lower bound becomes `>=` |
| `"polars[sql]>=1.20"` | `"polars[sql]>=1.21.0"` | extras are preserved |
| `"polars==1.20.0"` | unchanged | an exact pin is respected |
| `"polars<=1.20"` | unchanged | an upper-bound pin is respected |

To let a specific package move to a new major version, widen its
declaration's upper bound (or remove the `==` pin) and run `uu` again.

## Workspaces

If `uv.lock` has a `[manifest]` table with `members`, `uu` treats the project
as a uv workspace: it looks every member up in the lock's `[[package]]` rows
and follows that row's `source` — `editable` preferred over `virtual` — to the
member's directory, then bumps that member's `pyproject.toml` too. The root
file the `--pyproject-toml` option names decides which lock is used, and only
that one project tree is touched.

## The verbose table

`-v` prints, per processed file, one row per moved package:

```
	Package		Lock file		pyproject.toml
	python-dateutil		-		2.0 → 3.0.0
```

The *Lock file* column compares the before/after snapshots of the lock; a `-`
means the lock already held that version and only the manifest was stale —
which happens when uv failed to (or did not need to) move the package. The
*pyproject.toml* column always shows the manifest's old minimum bound
becoming the lock's new version. A package row appears only when the bump
actually changed a line.

## Origin

`uu` is a Rust port of [`uv-bump`](https://github.com/zundertj/uv-bump)
(zundertj, MIT), a Python tool. Faithful in spirit exactly where it matters —
which specifiers move, what the verbose table prints, that `pyproject.toml`
files keep their formatting — but without a Python installation, and with
Rust's sub-millisecond startup.
