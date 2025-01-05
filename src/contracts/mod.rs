//! The contracts module provides functionality to define and verify [Contract]s.

use crate::{ImportsInfo, PackageItemToken};
use anyhow::Result;
use std::collections::HashSet;

pub mod layers;

/// A contract defines a set of verifiable conditions
/// related to package imports that we wish to enforce.
pub trait Contract {
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
#[derive(Debug, Clone, PartialEq)]
pub struct ForbiddenImport {
    /// The start of the forbidden import path.
    pub from: PackageItemToken,
    /// The end of the forbidden import path.
    pub to: PackageItemToken,
    /// Items by which the import path is allowed.
    pub except_via: HashSet<PackageItemToken>,
}

impl ForbiddenImport {
    /// Creates a new [ForbiddenImport] from `from` to `to`.
    pub fn new(from: PackageItemToken, to: PackageItemToken) -> Self {
        ForbiddenImport {
            from,
            to,
            except_via: HashSet::new(),
        }
    }

    /// Allows imports via the passed items.
    pub fn except_via(mut self, items: &HashSet<PackageItemToken>) -> Self {
        self.except_via.extend(items);
        self
    }
}
