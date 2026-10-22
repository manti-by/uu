//! `uu` bumps the dependency minimum versions in `pyproject.toml` to the
//! versions `uv.lock` resolved to.
//!
//! The binary wiring lives in `src/main.rs`; everything testable is here.

pub mod bump;
pub mod error;
pub mod lock;
pub mod pyproject;
pub mod uv;
