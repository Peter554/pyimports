#![doc = include_str!("../README.md")]

mod errors;
mod imports_info;
mod package_info;
mod utils;

// TODO: Use #[cfg(test)] here, but still need
// a way to access the testutils from doctests.
// Related [GH issue](https://github.com/rust-lang/rust/issues/67295).
mod testutils;
pub use testutils::TestPackage;

pub use errors::Error;
pub use imports_info::{ImportMetadata, ImportsInfo, InternalImportsQueries, PackageItemTarget};
pub use package_info::{
    Module, ModuleToken, Package, PackageInfo, PackageItem, PackageItemToken, PackageToken,
};
