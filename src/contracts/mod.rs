//! The `contracts` module provides functionality to define and verify [`ImportsContract`]s.

use crate::imports_info::ImportsInfo;
use crate::package_info::PackageItemToken;
use crate::pypath::Pypath;
use anyhow::Result;
use derive_more::{IsVariant, Unwrap};
use derive_new::new;
use getset::{CopyGetters, Getters};
use std::collections::HashSet;

pub mod forbidden_external;
pub mod forbidden_internal;
pub mod independent;
pub mod layers;
mod utils;

/// An [`ImportsContract`] defines a set of verifiable conditions
/// related to imports that we wish to enforce.
pub trait ImportsContract {
    /// Verify the contract.
    fn verify(&self, imports_info: &ImportsInfo) -> Result<ContractVerificationResult>;
}

/// The result of verifying a contract.
#[derive(Debug, Clone, PartialEq, IsVariant, Unwrap)]
pub enum ContractVerificationResult {
    /// The contract was kept.
    Kept,
    /// The contract was violated. A vector of sample violations is returned.
    /// The returned violations are not guaranteed to be fully exhaustive - this is up to the
    /// specific contract implementation.
    Violated(Vec<ContractViolation>),
}

/// A violation of a contract.
#[derive(Debug, Clone, PartialEq)]
pub enum ContractViolation {
    /// An internal import which is forbidden by the contract.
    ForbiddenInternalImport {
        /// The import which is forbidden by the contract.
        forbidden_import: ForbiddenInternalImport,
        /// The specific path for this forbidden import.
        path: Vec<PackageItemToken>,
    },
    /// An external import which is forbidden by the contract.
    ForbiddenExternalImport {
        /// The import which is forbidden by the contract.
        forbidden_import: ForbiddenExternalImport,
        /// The specific path for this forbidden import.
        path: (Vec<PackageItemToken>, Pypath),
    },
}

/// An internal import which is forbidden.
#[derive(Debug, Clone, PartialEq, new, Getters, CopyGetters)]
pub struct ForbiddenInternalImport {
    /// The start of the forbidden import path.
    #[getset(get_copy = "pub")]
    from: PackageItemToken,
    /// The end of the forbidden import path.
    #[getset(get_copy = "pub")]
    to: PackageItemToken,
    /// Items by which the import path is allowed.
    /// E.g. if imports from `pkg.a` to `pkg.b` are forbidden, except via `pkg.c` then
    /// `pkg.a -> pkg.d -> pkg.b` would be a forbidden import path, while
    /// `pkg.a -> pkg.c -> pkg.b` would be allowed.
    #[new(into)]
    #[getset(get = "pub")]
    except_via: HashSet<PackageItemToken>,
}

/// An external import which is forbidden.
#[derive(Debug, Clone, PartialEq, new, Getters, CopyGetters)]
pub struct ForbiddenExternalImport {
    /// The start of the forbidden import path.
    #[getset(get_copy = "pub")]
    from: PackageItemToken,
    /// The end of the forbidden import path.
    #[getset(get = "pub")]
    to: Pypath,
    /// Items by which the import path is allowed.
    /// E.g. if imports from `pkg.a` to `pkg.b` are forbidden, except via `pkg.c` then
    /// `pkg.a -> pkg.d -> pkg.b` would be a forbidden import path, while
    /// `pkg.a -> pkg.c -> pkg.b` would be allowed.
    #[new(into)]
    #[getset(get = "pub")]
    except_via: HashSet<PackageItemToken>,
}
