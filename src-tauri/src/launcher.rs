//! 进程级操作。设计依据 DESIGN.md §1/§3。

#[path = "launcher_npm.rs"]
mod launcher_npm;
#[path = "launcher_runtime.rs"]
mod launcher_runtime;
#[cfg(test)]
#[path = "launcher_tests.rs"]
mod launcher_tests;

pub use launcher_npm::{install_npm, remove_version_dir, validate_dev_repo};
pub use launcher_runtime::{expand_tilde, fingerprint, home_dir, resolve_npm, DEV_BIN};
