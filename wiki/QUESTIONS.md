# Questions

Open questions worth revisiting; moved out of open chat threads so they don't
rot. Answered questions land in `pages/` as dated decision records.

- Should `uu` grow a `--check` mode that exits non-zero when the manifest is
  out of sync with the lock, so CI can assert the manifest has been kept up
  before anything downstream runs?
- Should workspace member detection also read the `[tool.uv.workspace]`
  section of `pyproject.toml`, as a fallback when the lock has no
  `[manifest]` block?
- Is the two-space gap in the verbose table worth aligning columns, or does
  upstream's exact tab layout win for parity forever?
