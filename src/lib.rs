#![doc = include_str!("../README.md")]

mod errors;
mod imports_info;
mod package_info;
mod utils;

// TODO: Use #[cfg(test)] here, but still need
// a way to access the testutils from doctests.
pub mod testutils;

pub use errors::Error;
pub use imports_info::{ImportMetadata, ImportsInfo, InternalImportsQueries};
pub use package_info::{
    Module, ModuleToken, Package, PackageInfo, PackageItem, PackageItemToken, PackageToken,
};
