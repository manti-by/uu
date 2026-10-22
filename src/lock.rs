//! Reading `uv.lock`: the package versions, and where every workspace member
//! lives.

use std::path::{Path, PathBuf};

use crate::error::Result;
use serde::Deserialize;

pub const LOCK_FILE_NAME: &str = "uv.lock";
pub const PYPROJECT_FILE_NAME: &str = "pyproject.toml";

/// The subset of `uv.lock` we care about. Everything else in the lock is
/// ignored, so the schema can lag behind uv's newest additions.
#[derive(Debug, Deserialize)]
pub struct Lock {
    #[serde(default)]
    package: Vec<Package>,
    manifest: Option<Manifest>,
}

#[derive(Debug, Deserialize)]
pub struct Package {
    name: String,
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<toml::Table>,
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    members: Option<Vec<String>>,
}

/// Parses lock file text.
pub fn parse_lock(contents: &str) -> Result<Lock> {
    Ok(toml::from_str(contents)?)
}

/// Loads the lock at `path` and returns its package versions as
/// `(name, version)` pairs.
///
/// Package rows without a `version` (they exist for workspace members that
/// only appear as dependencies) are skipped. Insertion order of the lock is
/// kept, because it is what the verbose table reports; a repeat of a name
/// updates the recorded version in place, as Python's dict would.
pub fn collect_package_versions(contents: &str) -> Result<Vec<(String, String)>> {
    let lock = parse_lock(contents)?;
    let mut versions: Vec<(String, String)> = Vec::new();
    for package in &lock.package {
        let Some(version) = &package.version else {
            continue;
        };
        match versions.iter_mut().find(|(name, _)| *name == package.name) {
            Some(slot) => slot.1 = version.clone(),
            None => versions.push((package.name.clone(), version.clone())),
        }
    }
    Ok(versions)
}

/// Determines every `pyproject.toml` this workspace owns, given the parsed
/// lock.
///
/// With a `[manifest]` block, one path per member is produced by looking the
/// member up in `[[package]]` and preferring an `editable` source over a
/// `virtual` one. Without one, the single project `pyproject.toml` next to the
/// lock is returned.
pub fn collect_pyproject_paths(contents: &str, lock_dir: &Path) -> Result<Vec<PathBuf>> {
    let lock = parse_lock(contents)?;

    let Some(members) = lock_manifest_members(&lock) else {
        return Ok(vec![lock_dir.join(PYPROJECT_FILE_NAME)]);
    };

    let mut paths = Vec::new();
    for member in members {
        let Some(package) = lock.package.iter().find(|p| p.name == *member) else {
            continue;
        };
        // Mirror the upstream lookup order: an editable source wins over a
        // virtual one, and only the first hit produces a path.
        for source_kind in ["editable", "virtual"] {
            if let Some(toml::Value::String(path)) = package
                .source
                .as_ref()
                .and_then(|source| source.get(source_kind))
            {
                paths.push(lock_dir.join(path).join(PYPROJECT_FILE_NAME));
                break;
            }
        }
    }
    Ok(paths)
}

fn lock_manifest_members(lock: &Lock) -> Option<&[String]> {
    lock.manifest
        .as_ref()
        .and_then(|manifest| manifest.members.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_versions_in_lock_order() {
        let versions = collect_package_versions(
            "[[package]]\nname = \"uv-bump\"\nversion = \"0.1.2\"\n\n\
             [[package]]\nname = \"polars\"\nversion = \"1.21.0\"\n",
        )
        .unwrap();
        assert_eq!(
            versions,
            vec![
                ("uv-bump".to_string(), "0.1.2".to_string()),
                ("polars".to_string(), "1.21.0".to_string()),
            ]
        );
    }

    #[test]
    fn repeats_update_in_place_not_order() {
        let versions = collect_package_versions(
            "[[package]]\nname = \"a\"\nversion = \"1.0\"\n\n\
             [[package]]\nname = \"b\"\nversion = \"2.0\"\n\n\
             [[package]]\nname = \"a\"\nversion = \"3.0\"\n",
        )
        .unwrap();
        assert_eq!(
            versions,
            vec![
                ("a".to_string(), "3.0".to_string()),
                ("b".to_string(), "2.0".to_string()),
            ]
        );
    }

    #[test]
    fn rows_without_version_are_skipped() {
        let versions = collect_package_versions("[[package]]\nname = \"a\"\n").unwrap();
        assert!(versions.is_empty());
    }

    #[test]
    fn workspaces_yield_member_paths_prefer_editable() {
        let resources = Path::new("tests/resources");
        let contents = std::fs::read_to_string(resources.join("uv_workspaces.lock")).unwrap();
        let paths = collect_pyproject_paths(&contents, resources).unwrap();
        assert_eq!(
            paths,
            vec![
                resources.join("pyproject.toml"),
                resources.join("packages/my_lib/pyproject.toml"),
            ]
        );
    }

    #[test]
    fn manifest_without_members_returns_single_file() {
        let contents = "version = 1\n\
                        requires-python = \">=3.11\"\n\n\
                        [manifest]\n\
                        overrides = [{ name = \"django\", specifier = \">=6.0.1\" }]\n\n\
                        [[package]]\nname = \"my-project\"\nversion = \"0.1.0\"\n\
                        source = { virtual = \".\" }\n";
        let paths = collect_pyproject_paths(contents, Path::new("some/dir")).unwrap();
        assert_eq!(paths, vec![PathBuf::from("some/dir/pyproject.toml")]);
    }
}
