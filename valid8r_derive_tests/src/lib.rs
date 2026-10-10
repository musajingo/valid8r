//! Executable examples from the workspace's READMEs.
//!
//! The derive crate's examples need the runtime crate, and Delta examples need
//! the Git dependency. Those dependencies are available in this private crate.
//! Run its doctests with `--include-ignored` to check every README example.

#[doc = include_str!("../../README.md")]
pub mod workspace_readme {}

#[doc = include_str!("../../valid8r/README.md")]
pub mod runtime_readme {}

#[doc = include_str!("../../valid8r_derive/README.md")]
pub mod derive_readme {}
