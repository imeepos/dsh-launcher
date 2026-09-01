//! 进程级操作。设计依据 DESIGN.md §1/§3。

#[path = "launcher_fingerprint.rs"]
mod launcher_fingerprint;
#[path = "launcher_npm.rs"]
mod launcher_npm;
#[cfg(test)]
#[path = "launcher_proc_tests.rs"]
mod launcher_proc_tests;
#[path = "launcher_runtime.rs"]
mod launcher_runtime;
#[cfg(test)]
#[path = "launcher_tests.rs"]
mod launcher_tests;

pub use launcher_fingerprint::{fingerprint, fingerprint_with_timeout};
pub use launcher_npm::{
    install_npm, install_npm_into, remove_version_dir, resolve_npm, validate_dev_repo,
};
pub use launcher_runtime::{expand_tilde, home_dir, DEV_BIN};
pub(crate) use launcher_runtime::{make_group_leader, strip_dsh_env};
