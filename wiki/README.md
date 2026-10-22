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
