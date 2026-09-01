//! ~/.dsh-launcher/registry.json 的数据模型与读写。

#[path = "registry_io.rs"]
mod registry_io;
#[path = "registry_methods.rs"]
mod registry_methods;
#[path = "registry_model.rs"]
mod registry_model;
#[cfg(test)]
#[path = "registry_tests.rs"]
mod registry_tests;

pub use registry_io::{
    default_registry_path, launcher_base_dir, load, now_ms, save, validate_id, versions_dir,
};
pub use registry_methods::*;
pub use registry_model::*;
