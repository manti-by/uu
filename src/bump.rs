//! The core operation: sync the lock, then rewrite every `pyproject.toml` the
//! workspace owns so its minimum bounds match what the lock resolved to.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::lock::{LOCK_FILE_NAME, PYPROJECT_FILE_NAME};
/// Bumps the minimum dependency versions in the root `pyproject.toml` (and in
/// every workspace member) to the versions the lock file holds.
///
/// Progress goes to `progress` (stderr in the binary): the checking line with
/// its dot animation, then either one line per updated package or the
/// everything-is-current line. `out` (stdout in the binary) receives the
/// verbose tables.
///
/// `sync` is the resolver step passed in by the caller — `uv::sync` in the
/// binary, a fixture in tests. It is handed the `upgrade` flag and is allowed
/// to rewrite the lock file: the versions around it are re-read as before and
/// after snapshots, which is what the verbose table reports.
pub fn bump(
    root_pyproject_toml_file: Option<PathBuf>,
    verbose: bool,
    upgrade: bool,
    sync: impl FnOnce(bool) -> Result<()>,
    out: &mut dyn Write,
    progress: &mut dyn Write,
) -> Result<()> {
    let root_pyproject_toml_file = root_pyproject_toml_file.unwrap_or_else(default_pyproject_path);
    let lock_path = root_pyproject_toml_file
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(LOCK_FILE_NAME);

    let lock_before = collect_package_versions_at(&lock_path)?;

    let checking = crate::progress::start(progress, crate::progress::stderr_is_terminal())?;
    let sync_result = sync(upgrade);
    checking.finish(progress)?;
    sync_result?;

    let lock_after = collect_package_versions_at(&lock_path)?;

    let contents = read_text(&lock_path)?;
    let pyproject_files = crate::lock::collect_pyproject_paths(
        &contents,
        lock_path.parent().unwrap_or_else(|| Path::new(".")),
    )?;

    // What moved, per package: the old version it is leaving behind, and the
    // new lock version. A pyproject bound that moved wins as the "old" value,
    // because that is what the user actually declared; a lock-only movement
    // falls back to the before snapshot. First sighting of a package wins.
    let mut updates: Vec<(String, String, String)> = Vec::new();
    // One entry per pyproject file: its path, the packages it updated, and
    // each updated package's prior version — the verbose table's inputs.
    type FileChanges = (PathBuf, Vec<String>, Vec<(String, String)>);
    let mut processed: Vec<FileChanges> = Vec::new();

    for pyproject_file in pyproject_files {
        let before_contents = read_text(&pyproject_file)?;
        let (contents_updated, packages_updated, before_versions) =
            crate::pyproject::update_contents(&before_contents, &lock_after);
        fs::write(&pyproject_file, contents_updated)?;

        for (package, old_version) in &before_versions {
            let new_version = version_of(&lock_after, package);
            bind_update(&mut updates, package, old_version, new_version);
        }
        processed.push((pyproject_file, packages_updated, before_versions));
    }

    for (package, version) in &lock_after {
        let before_version = version_of(&lock_before, package);
        if before_version != version {
            bind_update(&mut updates, package, before_version, version);
        }
    }

    if updates.is_empty() {
        writeln!(progress, "Everything up to date.")?;
    } else {
        for (package, old_version, new_version) in &updates {
            writeln!(progress, "  {package} {old_version} → {new_version}")?;
        }
    }

    for (pyproject_file, packages_updated, before_versions) in &processed {
        if verbose {
            writeln!(out, "Processed {}", pyproject_file.display())?;
            print_changes_table(
                before_versions,
                packages_updated,
                &lock_before,
                &lock_after,
                out,
            )?;
        }
    }

    Ok(())
}

/// Records that `package` moved from `old_version` to `new_version`, unless a
/// previous loop already reported this package.
fn bind_update(
    updates: &mut Vec<(String, String, String)>,
    package: &str,
    old_version: &str,
    new_version: &str,
) {
    if updates.iter().any(|(name, _, _)| name == package) {
        return;
    }
    updates.push((
        package.to_owned(),
        old_version.to_owned(),
        new_version.to_owned(),
    ));
}

fn default_pyproject_path() -> PathBuf {
    PathBuf::from(PYPROJECT_FILE_NAME)
}

/// Reads a file, naming it in the error so a missing `uv.lock` or
/// `pyproject.toml` is reported by its location, not just an os error.
fn read_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|error| {
        crate::error::Error::with_source(format!("could not read {}", path.display()), error)
    })
}

/// Reads and parses the lock at `path`.
fn collect_package_versions_at(path: &Path) -> Result<Vec<(String, String)>> {
    crate::lock::collect_package_versions(&read_text(path)?)
}

/// Looks a package up in a `(name, version)` list; `?` when absent.
fn version_of<'a>(versions: &'a [(String, String)], package: &str) -> &'a str {
    versions
        .iter()
        .find(|(name, _)| name == package)
        .map_or("?", |(_, version)| version)
}

/// Prints a table showing the lock file changes and, per `pyproject.toml`,
/// what its minimum bounds became.
fn print_changes_table(
    before_versions: &[(String, String)],
    packages_updated: &[String],
    lock_before: &[(String, String)],
    lock_after: &[(String, String)],
    out: &mut dyn Write,
) -> Result<()> {
    writeln!(out, "\tPackage\t\tLock file\t\tpyproject.toml")?;

    for package in packages_updated {
        let lock_b = version_of(lock_before, package);
        let lock_a = version_of(lock_after, package);
        let pyproject_b = version_of(before_versions, package);

        let lock_str = if lock_b != lock_a {
            format!("{lock_b} → {lock_a}")
        } else {
            "-".to_owned()
        };
        let pyproject_str = format!("{pyproject_b} → {lock_a}");

        writeln!(out, "\t{package}\t\t{lock_str}\t\t{pyproject_str}")?;
    }

    if packages_updated.is_empty() {
        writeln!(out, "\tNo packages updated")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(name: &str, version: &str) -> (String, String) {
        (name.to_owned(), version.to_owned())
    }

    fn versions() -> Vec<(String, String)> {
        vec![pair("uv-bump", "0.1.2"), pair("polars", "1.20.0")]
    }

    #[test]
    fn header_printed_even_without_updates() {
        let mut out: Vec<u8> = Vec::new();
        print_changes_table(&Vec::new(), &[], &Vec::new(), &Vec::new(), &mut out).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "\tPackage\t\tLock file\t\tpyproject.toml\n\tNo packages updated\n"
        );
    }

    #[test]
    fn lock_line_uses_arrow_when_version_moved() {
        let mut out: Vec<u8> = Vec::new();
        let before = versions();
        let after = vec![pair("uv-bump", "0.1.2"), pair("polars", "1.21.0")];
        print_changes_table(
            &[pair("polars", "1.20.0")],
            &["polars".to_owned()],
            &before,
            &after,
            &mut out,
        )
        .unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("\tpolars\t\t1.20.0 → 1.21.0"));
        assert!(text.contains("1.20.0 → 1.21.0"));
    }

    #[test]
    fn lock_line_uses_dash_when_lock_did_not_move() {
        // The lock held 1.21.0 all along; only pyproject was stale.
        let mut out: Vec<u8> = Vec::new();
        let lock = vec![pair("uv-bump", "0.1.2"), pair("polars", "1.21.0")];
        print_changes_table(
            &[pair("polars", "1.20.0")],
            &["polars".to_owned()],
            &lock,
            &lock,
            &mut out,
        )
        .unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("\tpolars\t\t-\t\t1.20.0 → 1.21.0"));
    }

    #[test]
    fn unknown_row_reports_question_mark() {
        assert_eq!(version_of(&versions(), "duckdb"), "?");
    }

    #[test]
    fn default_pyproject_used_without_explicit_root() {
        assert_eq!(default_pyproject_path(), PathBuf::from("pyproject.toml"));
    }

    #[test]
    fn sync_closure_receives_the_upgrade_flag() {
        let dir = std::env::temp_dir().join(format!("uu-bump-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("uv.lock"),
            "[[package]]\nname = \"polars\"\nversion = \"1.21.0\"\n",
        )
        .unwrap();
        fs::write(
            dir.join("pyproject.toml"),
            "[project]\ndependencies = [\"polars>=1.20.0\"]\n",
        )
        .unwrap();

        let mut out: Vec<u8> = Vec::new();
        let mut progress: Vec<u8> = Vec::new();
        let mut seen = false;
        bump(
            Some(dir.join("pyproject.toml")),
            false,
            true,
            |upgrade| {
                seen = upgrade;
                Ok(())
            },
            &mut out,
            &mut progress,
        )
        .unwrap();
        assert!(seen);

        let pyproject = fs::read_to_string(dir.join("pyproject.toml")).unwrap();
        assert!(pyproject.contains("polars>=1.21.0"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn status_reports_the_updated_package() {
        let dir = std::env::temp_dir().join(format!("uu-bump-status-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("uv.lock"),
            "[[package]]\nname = \"polars\"\nversion = \"1.21.0\"\n",
        )
        .unwrap();
        fs::write(
            dir.join("pyproject.toml"),
            "[project]\ndependencies = [\"polars>=1.20.0\"]\n",
        )
        .unwrap();

        let mut out: Vec<u8> = Vec::new();
        let mut progress: Vec<u8> = Vec::new();
        bump(
            Some(dir.join("pyproject.toml")),
            false,
            false,
            |_| Ok(()),
            &mut out,
            &mut progress,
        )
        .unwrap();

        let text = String::from_utf8(progress).unwrap();
        assert!(text.starts_with("Checking updates..."));
        assert!(text.contains("  polars 1.20.0 → 1.21.0"));
        assert!(!text.contains("Everything up to date"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn no_change_reports_up_to_date() {
        let dir = std::env::temp_dir().join(format!("uu-bump-clean-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("uv.lock"),
            "[[package]]\nname = \"polars\"\nversion = \"1.21.0\"\n",
        )
        .unwrap();
        fs::write(
            dir.join("pyproject.toml"),
            "[project]\ndependencies = [\"polars>=1.21.0\"]\n",
        )
        .unwrap();

        let mut out: Vec<u8> = Vec::new();
        let mut progress: Vec<u8> = Vec::new();
        bump(
            Some(dir.join("pyproject.toml")),
            false,
            false,
            |_| Ok(()),
            &mut out,
            &mut progress,
        )
        .unwrap();

        let text = String::from_utf8(progress).unwrap();
        assert!(text.contains("Everything up to date."), "\n{text}");
        assert!(!text.contains(" → "), "no update lines expected:\n{text}");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn lock_only_movement_is_reported() {
        // The lock moved but the manifest had no feasible bound for the
        // package; the update line falls back to the before snapshot.
        let dir = std::env::temp_dir().join(format!("uu-bump-lockonly-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let before_lock = "[[package]]\nname = \"pytest\"\nversion = \"8.3.4\"\n";
        let after_lock = "[[package]]\nname = \"pytest\"\nversion = \"8.4.0\"\n";
        fs::write(dir.join("uv.lock"), before_lock).unwrap();
        fs::write(dir.join("pyproject.toml"), "[project]\ndependencies = []\n").unwrap();

        let mut progress: Vec<u8> = Vec::new();
        let mut out: Vec<u8> = Vec::new();

        let lock_dir = dir.clone();
        bump(
            Some(dir.join("pyproject.toml")),
            false,
            false,
            |_| {
                fs::write(lock_dir.join("uv.lock"), after_lock).unwrap();
                Ok(())
            },
            &mut out,
            &mut progress,
        )
        .unwrap();

        let text = String::from_utf8(progress).unwrap();
        assert!(text.contains("  pytest 8.3.4 → 8.4.0"), "\n{text}");
        fs::remove_dir_all(&dir).unwrap();
    }
}
