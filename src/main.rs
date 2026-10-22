//! `uu` bumps the dependency minimum versions in `pyproject.toml` to the
//! versions `uv.lock` resolved to, the way `uv sync --upgrade` would leave
//! them if uv also edited the manifest.
//!
//! Argument parsing and dispatch only; the logic lives in `src/lib.rs`.

use std::path::PathBuf;
use std::process::ExitCode;

use uu::{bail, bump, error::Result, uv};

const USAGE: &str = "\
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
";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Parses arguments and runs the bump.
fn run(args: Vec<String>) -> Result<()> {
    let mut pyproject_toml: Option<PathBuf> = None;
    let mut verbose = false;
    // Without `--no-upgrade`, `uv sync` re-resolves to newest versions, which
    // is what puts new version numbers into the lock in the first place.
    let mut upgrade = true;

    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        match arg {
            "-h" | "--help" | "help" => {
                print!("{USAGE}");
                return Ok(());
            }
            "-V" | "--version" => {
                println!("uu {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "-v" | "--verbose" => verbose = true,
            "-n" | "--no-upgrade" => upgrade = false,
            // Both the separated and the `=`-joined form of a value option are
            // accepted, so a following option is never mistaken for the file
            // name: a value may not start with a dash.
            "--pyproject-toml" => {
                pyproject_toml = Some(value_at(&args, index)?);
                index += 1;
            }
            other if other.starts_with("--pyproject-toml=") => {
                pyproject_toml = Some(PathBuf::from(
                    other
                        .strip_prefix("--pyproject-toml=")
                        .expect("checked above"),
                ));
            }
            other if other.starts_with('-') => {
                bail!("unknown option `{other}`; run `uu --help`")
            }
            _ => bail!("unexpected argument `{arg}`; options only, run `uu --help`"),
        }
        index += 1;
    }

    // Stdout and Stderr lock per write call, so they must be passed as the
    // streams themselves and not as a held lock: the spinner thread locks
    // stderr while the main thread writes through it, and a lock held for the
    // whole run would deadlock the animation.
    let mut out = std::io::stdout();
    let mut progress = std::io::stderr();
    bump::bump(
        pyproject_toml,
        verbose,
        upgrade,
        uv::sync,
        &mut out,
        &mut progress,
    )
}

/// Reads the value of the `--pyproject-toml` option at `index`, requiring the
/// following argument.
fn value_at(args: &[String], index: usize) -> Result<PathBuf> {
    match args.get(index + 1) {
        // Reject a following option masquerading as the value, so a typo like
        // `uu --pyproject-toml -v` does not bump a file named `-v`.
        Some(value) if !value.starts_with('-') => Ok(PathBuf::from(value)),
        _ => bail!("missing value for --pyproject-toml"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_and_version_do_not_error() {
        run(vec!["--help".to_owned()]).unwrap();
        run(vec!["-V".to_owned()]).unwrap();
        run(vec!["-h".to_owned()]).unwrap();
        run(vec!["--version".to_owned()]).unwrap();
        run(vec!["help".to_owned()]).unwrap();
    }

    #[test]
    fn unknown_option_is_rejected() {
        let error = run(vec!["--wat".to_owned()]).unwrap_err();
        assert!(error.to_string().contains("unknown option `--wat`"));
    }

    #[test]
    fn positional_arguments_are_rejected() {
        let error = run(vec!["here".to_owned()]).unwrap_err();
        assert!(error.to_string().contains("unexpected argument"));
    }

    #[test]
    fn value_flag_requires_a_value() {
        let error = run(vec!["--pyproject-toml".to_owned()]).unwrap_err();
        assert!(error.to_string().contains("missing value"));
        let error = run(vec!["--pyproject-toml".to_owned(), "-v".to_owned()]).unwrap_err();
        assert!(error.to_string().contains("missing value"));
    }

    #[test]
    fn inline_value_form_is_accepted() {
        // Parsing stops before touching any file; a missing file errors, which
        // still proves the value was parsed out of the `=` form.
        let error = run(vec!["--pyproject-toml=x/pyproject.toml".to_owned()]).unwrap_err();
        assert!(error.to_string().contains("No such file"));
    }
}
