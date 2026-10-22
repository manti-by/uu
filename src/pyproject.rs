//! Bumping `>=`, `~=` and `>` minimum bounds in a `pyproject.toml` text.
//!
//! The upstream tool edits the file with a single regex per package, so
//! comments, spacing and unrelated tables survive untouched. That behaviour is
//! reproduced exactly here rather than reparsing the TOML.

use regex::Regex;

/// Replaces every feasible minimum bound of `package` in `text` with an exact
/// `>= version` specifier.
///
/// A bound is feasible unless the declaration pins the version (`==`, `<=`, or
/// carries an upper bound as a later specifier) — an upper bound survives
/// because only the first specifier is rewritten. Returns the new text, the
/// number of lines that differ, and the previous bound's bare version segment.
pub fn replace_package_version(
    text: &str,
    package: &str,
    version: &str,
) -> (String, usize, Option<String>) {
    // We assume the following:
    // 1. the package name is directly preceded by a double quote
    // 2. `(?:\[[^\]]*\])?` after the package name there can be extras, if so,
    //    we require it to be in square brackets
    // 3. `(>=|~=|>)` there will be a version specifier that allows updating
    // 4. `[^"`,;]+` we need to stop. There can be more version specifiers,
    //    separated by comma, and/or system specifiers (separated by semicolon),
    //    or none, in which case we encounter the double quotes.
    let pattern = Regex::new(&format!(
        r#""({}(?:\[[^\]]*\])?)\s*(>=|~=|>)\s*([^"`,;]+)"#,
        regex::escape(package)
    ))
    .expect("package name escaped before building the regex");

    let before_version = pattern
        .captures(text)
        .map(|captures| captures[3].trim().to_owned());

    // The replacement is the opening quote, the package (with any extras), and
    // the pinned lower bound; everything after — further specifiers, comments
    // — stays as it was.
    let updated = pattern
        .replace_all(text, format!("\"$1>={version}"))
        .into_owned();

    // We count the number of lines changed, rather than the number of
    // replacements, as a replacement may not change the text at all (same
    // version). The edit never changes the line count.
    let count = updated
        .lines()
        .zip(text.lines())
        .filter(|(new, old)| new != old)
        .count();

    (updated, count, before_version)
}

/// Applies every package/version pair in `versions` to a `pyproject.toml`
/// text.
///
/// Returns the new text, the packages that moved, and each moved package's
/// previous version.
pub fn update_contents(
    contents: &str,
    versions: &[(String, String)],
) -> (String, Vec<String>, Vec<(String, String)>) {
    let mut contents = contents.to_owned();
    let mut packages_updated = Vec::new();
    let mut before_versions = Vec::new();

    for (package, version) in versions {
        let (updated, count, before_version) = replace_package_version(&contents, package, version);
        contents = updated;
        if count > 0 {
            packages_updated.push(package.clone());
            if let Some(before_version) = before_version {
                before_versions.push((package.clone(), before_version));
            }
        }
    }

    (contents, packages_updated, before_versions)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bump(contents: &str, versions: &[(&str, &str)]) -> String {
        let owned: Vec<(String, String)> = versions
            .iter()
            .map(|(p, v)| (p.to_string(), v.to_string()))
            .collect();
        update_contents(contents, &owned).0
    }

    #[test]
    fn updates_upper_bound_kept_intact() {
        let result = bump("\n    \"polars>=1.20.0,<1.22\",\n", &[("polars", "1.21.0")]);
        assert!(result.contains(r#""polars>=1.21.0,<1.22""#));
    }

    #[test]
    fn updates_extras() {
        let result = bump("\n    \"polars[sql]>=1.20\",\n", &[("polars", "1.21.0")]);
        assert!(result.contains(r#""polars[sql]>=1.21.0""#));
    }

    #[test]
    fn leaves_exact_pins_alone() {
        // This should not happen to start with, but good to check we only
        // modify parts where we expect an upgrade to be possible.
        let contents = "\n   \"polars==1.20.0\",\n";
        let result = bump(contents, &[("polars", "1.21.0")]);
        assert_eq!(result, contents);
    }

    #[test]
    fn leaves_upper_bound_pin_alone() {
        let contents = "\"polars<=1.20\",\n";
        assert_eq!(bump(contents, &[("polars", "1.21.0")]), contents);
    }

    #[test]
    fn keeps_inline_comment() {
        let contents = "\n    \"polars>=1.20.0\",  # 1.21 has a bug\n";
        let result = bump(contents, &[("polars", "1.21.0")]);
        assert!(result.contains("\"polars>=1.21.0\",  # 1.21 has a bug"));
    }

    #[test]
    fn rewrites_tilde_equal() {
        let result = bump("\n    \"polars~=1.20.0\",\n", &[("polars", "1.21.0")]);
        assert!(result.contains(r#""polars>=1.21.0""#));
    }

    #[test]
    fn tracks_before_version() {
        let owned = vec![("polars".to_string(), "1.21.0".to_string())];
        let (_, updated, before) = update_contents("\n    \"polars>=1.20.0,<1.22\",\n", &owned);
        assert_eq!(updated, vec!["polars".to_string()]);
        assert_eq!(before, vec![("polars".to_string(), "1.20.0".to_string())]);
    }

    #[test]
    fn regex_special_characters_in_name_are_literal() {
        let result = bump("\n    \"my-pkg+safe>=1.0\",\n", &[("my-pkg+safe", "2.0")]);
        assert!(result.contains(r#""my-pkg+safe>=2.0""#));
    }

    #[test]
    fn same_version_counts_as_no_change() {
        let owned = vec![("polars".to_string(), "1.20.0".to_string())];
        let (_, updated, _) = update_contents("\n    \"polars>=1.20.0\",\n", &owned);
        assert!(updated.is_empty());
    }

    #[test]
    fn comment_only_mention_is_untouched() {
        let contents = "# see polars>=1.0 docs\n";
        assert_eq!(bump(contents, &[("polars", "2.0")]), contents);
    }
}
