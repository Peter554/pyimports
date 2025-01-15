#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod contracts;

pub mod errors;
pub mod imports_info;
pub mod package_info;
pub mod parse;
pub mod pypath;

// TODO: Use #[cfg(test)] here, but still need
// a way to access the testutils from doctests.
// Related [GH issue](https://github.com/rust-lang/rust/issues/67295).
#[doc(hidden)]
#[cfg(feature = "testutils")]
pub mod testutils;

#[allow(dead_code)]
#[doc(hidden)]
#[cfg(feature = "grimp_compare")]
pub mod grimp_compare;

/// Extension traits used by pyimports.
///
/// ```
/// use pyimports::prelude::*;
/// ```
pub mod prelude {
    pub use crate::package_info::ExtendWithDescendants;
    pub use crate::package_info::PackageItemIterator;
}
