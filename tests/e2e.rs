//! End-to-end tests over the full `bump` pipeline, mirroring the upstream
//! `uv-bump` test suite: a fake resolver rewrites the lock file, and the bump
//! must bring `pyproject.toml` along.

use std::fs;
use std::path::{Path, PathBuf};

use uu::bump::bump;

const LOCK_BEFORE: &str = "\
[[package]]
name = \"uv-bump\"
version = \"0.1.2\"

[[package]]
name = \"polars\"
version = \"1.20.0\"
";

const LOCK_AFTER: &str = "\
[[package]]
name = \"uv-bump\"
version = \"0.1.2\"

[[package]]
name = \"polars\"
version = \"1.21.0\"
";

const PYPROJECT: &str = "\
[project]
name = \"uv-bump\"
version = \"0.1.2\"
description = \"Bump pyproject.toml dependency minimum versions.\"
readme = \"README.md\"
requires-python = \">=3.12\"
dependencies = [
    \"polars>=1.20.0,<1.22\",
]";

/// A temp directory holding the project: `pyproject.toml`, and a `uv.lock`
/// that the fake resolver rewrites. Both sinks capture output: `out` is what
/// the binary sends to stdout, `progress` is what it sends to stderr.
struct Project {
    root: PathBuf,
}

impl Project {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "uu-e2e-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&root).unwrap();
        let project = Self { root };
        project.write("pyproject.toml", PYPROJECT);
        project.write("uv.lock", LOCK_BEFORE);
        project
    }

    fn write(&self, name: &str, contents: &str) {
        fs::write(self.root.join(name), contents).unwrap();
    }

    fn read(&self, name: &str) -> String {
        fs::read_to_string(self.root.join(name)).unwrap()
    }

    /// The fake resolver: swaps the lock for the upgraded contents, exactly
    /// what `uv sync --upgrade` would leave behind.
    fn sync(&self) -> impl Fn(bool) -> uu::error::Result<()> {
        let root = self.root.clone();
        move |upgrade| {
            assert!(upgrade, "the default run must ask for --upgrade");
            fs::write(root.join("uv.lock"), LOCK_AFTER)?;
            Ok(())
        }
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn capture(out: Vec<u8>) -> String {
    String::from_utf8(out).unwrap()
}

#[test]
fn bumps_pyproject_to_lock_versions() {
    let project = Project::new();
    let mut out: Vec<u8> = Vec::new();
    let mut progress: Vec<u8> = Vec::new();

    let sync = project.sync();
    bump(
        Some(project.root.join("pyproject.toml")),
        false,
        true,
        sync,
        &mut out,
        &mut progress,
    )
    .unwrap();

    assert!(project.read("pyproject.toml").contains(
        "[project]\n\
         name = \"uv-bump\"\n\
         version = \"0.1.2\"\n\
         description = \"Bump pyproject.toml dependency minimum versions.\"\n\
         readme = \"README.md\"\n\
         requires-python = \">=3.12\"\n\
         dependencies = [\n    \"polars>=1.21.0,<1.22\",\n]"
    ));
}

#[test]
fn verbose_reports_both_snapshots() {
    // The pyproject is bumped while the lock was already at the new version,
    // so the lock column shows a dash and the pyproject column shows its own
    // movement.
    let project = Project::new();
    project.write("uv.lock", LOCK_AFTER);
    let mut out: Vec<u8> = Vec::new();
    let mut progress: Vec<u8> = Vec::new();

    let sync = project.sync();
    bump(
        Some(project.root.join("pyproject.toml")),
        true,
        true,
        sync,
        &mut out,
        &mut progress,
    )
    .unwrap();

    let text = capture(out);
    assert!(
        text.contains("Processed"),
        "verbose must name the file:\n{text}"
    );
    assert!(
        text.contains("\tpolars\t\t-\t\t1.20.0 → 1.21.0"),
        "expected the change row in:\n{text}"
    );
}

#[test]
fn verbose_reports_a_genuinely_updated_lock() {
    let project = Project::new();
    let mut out: Vec<u8> = Vec::new();
    let mut progress: Vec<u8> = Vec::new();

    let sync = project.sync();
    bump(
        Some(project.root.join("pyproject.toml")),
        true,
        true,
        sync,
        &mut out,
        &mut progress,
    )
    .unwrap();

    let text = capture(out);
    assert!(
        text.contains("\tpolars\t\t1.20.0 → 1.21.0"),
        "expected the lock movement in:\n{text}"
    );
}

#[test]
fn status_lines_report_what_moved() {
    let project = Project::new();
    let mut out: Vec<u8> = Vec::new();
    let mut progress: Vec<u8> = Vec::new();

    let sync = project.sync();
    bump(
        Some(project.root.join("pyproject.toml")),
        false,
        true,
        sync,
        &mut out,
        &mut progress,
    )
    .unwrap();

    let text = capture(progress);
    assert!(text.starts_with("Checking updates..."), "\n{text}");
    assert!(text.contains("  polars 1.20.0 → 1.21.0"), "\n{text}");
    assert!(!text.contains("Everything up to date"), "\n{text}");
}

#[test]
fn nothing_moving_says_up_to_date() {
    // Both snapshots agree and the manifest already holds the lock versions:
    // no line with an arrow may appear, only the summary.
    let project = Project::new();
    project.write("uv.lock", LOCK_AFTER);
    project.write(
        "pyproject.toml",
        "[project]\n\
         name = \"uv-bump\"\n\
         version = \"0.1.2\"\n\
         description = \"Bump pyproject.toml dependency minimum versions.\"\n\
         readme = \"README.md\"\n\
         requires-python = \">=3.12\"\n\
         dependencies = [\n    \"polars>=1.21.0\",\n]",
    );
    let mut out: Vec<u8> = Vec::new();
    let mut progress: Vec<u8> = Vec::new();

    let sync = project.sync();
    bump(
        Some(project.root.join("pyproject.toml")),
        false,
        true,
        sync,
        &mut out,
        &mut progress,
    )
    .unwrap();

    let text = capture(progress);
    assert!(text.starts_with("Checking updates..."), "\n{text}");
    assert!(text.contains("Everything up to date."), "\n{text}");
    assert!(!text.contains(" → "), "\n{text}");
}

#[test]
fn workspaces_bump_every_member() {
    let resources = Path::new("tests/resources");
    let root = std::env::temp_dir().join(format!(
        "uu-e2e-ws-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    fs::create_dir_all(root.join("packages/my_lib")).unwrap();

    fs::copy(resources.join("uv_workspaces.lock"), root.join("uv.lock")).unwrap();
    fs::write(
        root.join("pyproject.toml"),
        "[project]\nname = \"my-app\"\ndependencies = [\"pluggy>=1.0\"]\n",
    )
    .unwrap();
    fs::write(
        root.join("packages/my_lib/pyproject.toml"),
        "[project]\nname = \"my-lib\"\ndependencies = [\"pytest>=8.0\"]\n",
    )
    .unwrap();

    let mut out: Vec<u8> = Vec::new();
    let mut progress: Vec<u8> = Vec::new();
    bump(
        Some(root.join("pyproject.toml")),
        true,
        false,
        |_upgrade| Ok(()),
        &mut out,
        &mut progress,
    )
    .unwrap();

    // Every workspace member the lock names was visited.
    let text = capture(out);
    assert!(text.contains("packages/my_lib/pyproject.toml"), "\n{text}");

    // The status lines aggregate updates across every member file.
    let status = capture(progress);
    assert!(status.contains("  pytest 8.0 → 8.3.4"), "\n{status}");
    assert!(status.contains("  pluggy 1.0 → 1.5.0"), "\n{status}");
    assert!(
        fs::read_to_string(root.join("packages/my_lib/pyproject.toml"))
            .unwrap()
            .contains("pytest>=8.3.4")
    );
    assert!(
        fs::read_to_string(root.join("pyproject.toml"))
            .unwrap()
            .contains("pluggy>=1.5.0")
    );
    fs::remove_dir_all(&root).unwrap();
}
