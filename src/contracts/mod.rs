//! The contracts module provides functionality to define and verify [ImportsContract]s.

use crate::{ImportsInfo, PackageItemToken};
use anyhow::Result;
use derive_getters::Getters;
use derive_new::new;
use std::collections::HashSet;

pub mod layers;

/// An imports contract defines a set of verifiable conditions
/// related to imports that we wish to enforce.
pub trait ImportsContract {
    /// Verify the contract, returning a vector of violations.
    /// The violations are not guaranteed to be exhaustive - this is up to the
    /// specific contract implementation.
    fn verify(&self, imports_info: &ImportsInfo) -> Result<Vec<ContractViolation>>;
}

/// A violation of a contract.
#[derive(Debug, Clone, PartialEq)]
pub enum ContractViolation {
    /// An import path which is forbidden by the contract.
    ForbiddenImport {
        /// The import which is forbidden by the contract.
        forbidden_import: ForbiddenImport,
        /// The specific path for this forbidden import.
        path: Vec<PackageItemToken>,
    },
}

/// An import path which is forbidden.
#[derive(Debug, Clone, PartialEq, new, Getters)]
pub struct ForbiddenImport {
    /// The start of the forbidden import path.
    #[getter(copy)]
    from: PackageItemToken,
    /// The end of the forbidden import path.
    #[getter(copy)]
    to: PackageItemToken,
    /// Items by which the import path is allowed.
    #[new(into)]
    except_via: HashSet<PackageItemToken>,
}
