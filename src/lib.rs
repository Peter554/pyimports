#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod contracts;

mod errors;
mod imports_info;
mod package_info;
mod pypath;

// TODO: Use #[cfg(test)] here, but still need
// a way to access the testutils from doctests.
// Related [GH issue](https://github.com/rust-lang/rust/issues/67295).
// Could perhaps alternatively use a feature flag, disabled by default?
mod testutils;

#[doc(hidden)]
pub use testutils::TestPackage;

pub use errors::Error;
pub use imports_info::{
    ExplicitImportMetadata, ExternalImportsQueries, ImportMetadata, ImportsInfo,
    ImportsInfoBuildOptions, InternalImportsPathQuery, InternalImportsPathQueryBuilder,
    InternalImportsPathQueryBuilderError, InternalImportsQueries,
};
pub use package_info::{
    ExtendWithDescendants, Module, ModuleToken, Package, PackageInfo, PackageItem,
    PackageItemIterator, PackageItemToken, PackageToken,
};
pub use pypath::{IntoPypath, Pypath};

/// Extension traits used by pyimports.
///
/// ```
/// use pyimports::prelude::*;
/// ```
pub mod prelude {
    pub use crate::ExtendWithDescendants;
    pub use crate::IntoPypath;
    pub use crate::PackageItemIterator;
}
