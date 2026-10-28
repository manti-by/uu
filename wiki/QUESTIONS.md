# Questions

Open questions worth revisiting; moved out of open chat threads so they don't
rot. Answered questions land in `pages/` as dated decision records.

Newest first under `## Open`, numbered with the next unused `Q-NNN`. Do not
re-file a question that already exists, and leave `**Answer:**` blank for a
human to fill in. A filled-in answer gets applied to its `**Pages:**` and the
entry moves to `## Resolved` with a one-line note of what changed.

<!--
### Q-NNN — <short title>
**Date:** YYYY-MM-DD
**Pages:** `2026-10-22-page-name` (one per conflicting page, wrapped in wiki-link brackets)
**Discrepancy:** what conflicts, and what was checked that did not settle it.
**Answer:**

For a product question with no page conflict, use `**Question:**` in place of
`**Discrepancy:**` and keep `**Pages:** none — product question`.
-->

## Open

### Q-004 — Amend decision pages in place, or supersede them?

**Date:** 2026-10-28
**Pages:** [[2026-10-22-progress-output-and-tty-animation]], [[2026-10-22-uu-rust-port-and-repo-structure]]
**Discrepancy:** `wiki/README.md` says a page is written when a decision is
made, "not as a living document, and is not edited later; a later decision
superseding an older one writes a new page". The consistency pass asks for
outdated pages to be corrected in place with a note. This run read the two as
compatible by severity: the changed animation decision got its own page
(`2026-10-28-in-place-three-dot-animation`), while the port page's factual
error about where the specifier tests live was corrected in place with an
`Amended` note. Which reading should win when a page is merely wrong rather
than superseded?
**Answer:**

### Q-001 — Should `uu` grow a `--check` mode?

**Date:** 2026-10-22
**Pages:** none — product question
**Question:** Should `uu` exit non-zero when the manifest is out of sync with
the lock, so CI can assert the manifest has been kept up before anything
downstream runs?
**Answer:**

### Q-002 — Should workspace detection fall back to `[tool.uv.workspace]`?

**Date:** 2026-10-22
**Pages:** none — product question
**Question:** Should member detection also read the `[tool.uv.workspace]`
section of `pyproject.toml`, as a fallback when the lock has no `[manifest]`
block?
**Answer:**

### Q-003 — Align the verbose table's columns?

**Date:** 2026-10-22
**Pages:** none — product question
**Question:** Is the two-space gap in the verbose table worth aligning columns
for, or does upstream's exact tab layout win for parity forever?
**Answer:**

## Resolved

None yet. An answered entry moves here with the date it was applied and a
one-line note of what changed.
