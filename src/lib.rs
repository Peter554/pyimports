//! A rust crate for parsing and analyzing the imports within a python package.
//!
//! ```rust
//! use anyhow::Result;
//!
//! use pyimports::{testpackage,testutils::TestPackage};
//! use pyimports::{PackageInfo,ImportsInfo};
//!
//! fn main() -> Result<()> {
//!     let testpackage = testpackage! {
//!         "__init__.py" => "",
//!         "a.py" => ""
//!     };
//!
//!     let package_info = PackageInfo::build(testpackage.path())?;
//!     let imports_info = ImportsInfo::build(package_info)?;
//!
//!     Ok(())
//! }
//! ```

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
