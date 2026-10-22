//! Shelling out to `uv sync`, the resolver that decides what the versions in
//! `pyproject.toml` should become.

use std::process::Command;

use crate::error::{Error, Result};

/// The `uv sync` argument list: same as the upstream tool, extras installed so
/// every extras-bound dependency also resolves, plus `--upgrade` on request.
pub fn sync_args(upgrade: bool) -> Vec<&'static str> {
    let mut args = vec!["sync", "--all-extras"];
    if upgrade {
        args.push("--upgrade");
    }
    args
}

/// Runs `uv sync` capturing its output; a failing sync carries its exit code
/// and stderr back to the user, success stays silent like the upstream tool.
pub fn sync(upgrade: bool) -> Result<()> {
    let output = Command::new("uv").args(sync_args(upgrade)).output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let message = match output.status.code() {
                Some(code) => {
                    format!("uv sync failed (exit code {code}):\n{stderr}")
                }
                None => format!("uv sync was terminated by a signal:\n{stderr}"),
            };
            Err(Error::new(message.trim_end().to_string()))
        }
        Err(error) => Err(Error::with_source(
            format!("could not run `uv`; is uv installed? ({error})"),
            error,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrade_appends_the_flag() {
        assert_eq!(sync_args(true), vec!["sync", "--all-extras", "--upgrade"]);
    }

    #[test]
    fn no_upgrade_omits_the_flag() {
        assert_eq!(sync_args(false), vec!["sync", "--all-extras"]);
    }
}
