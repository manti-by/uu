# Wiki

The wiki holds the long-form documentation the README deliberately does not.

## Docs

| File | Contents | Owner of |
| --- | --- | --- |
| `docs/INSTALL.md` | Every way to install `uu`, from the one-line curl to `cargo install` and manual downloads. | Install. The README links here and never repeats the details. |
| `docs/MORE.md` | The complete command list and the pyproject specifier semantics: what moves, what never moves, workspaces, `uv sync`. | How it works. The README does not repeat this list. |

## Pages

`pages/` holds dated decision records — one file per significant design
decision, named `YYYY-MM-DD-topic.md`. A page is written when a decision was
made, not as a living document, and is not edited later; a later decision
superseding an older one writes a new page.

## Rules

- The README stays a short front door; everything that grows belongs here.
- Each fact lives in exactly one file. Documentation duplicated in two places
  drifts apart.
- Console transcripts in documentation are copied from a real run of the
  binary, never typed from memory. If a transcript has to be regenerated, run
  the command again.

## Questions and index

`QUESTIONS.md` holds open questions newest-first under `## Open`, numbered
`Q-NNN`. A human fills in `**Answer:**`; the answer is then applied to the
pages named in `**Pages:**`, and the entry moves to `## Resolved` with a note
of what changed. The entry shape lives in the file's comment block.

`INDEX.md` catalogs the wiki: `## Pages` lists every page once, newest first,
as `- [title](pages/file.md) — summary (date)`, and `## By topic` clusters the
same pages by subject. Both sections are rebuilt whenever a page is added,
retitled or corrected, so the index never drifts from `pages/`.
