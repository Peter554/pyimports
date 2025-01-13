//! The `forbidden_external` module provides a [`ForbiddenExternalImportContract`], which forbids a certain external import.
//!
//! # Example: Contract kept
//!
//! ```
//! # use anyhow::Result;
//! # use pyimports::{testpackage};
//! # use pyimports::testutils::TestPackage;
//! use pyimports::package_info::PackageInfo;
//! use pyimports::imports_info::ImportsInfo;
//! use pyimports::contracts::ImportsContract;
//! use pyimports::contracts::forbidden_external::ForbiddenExternalImportContract;
//!
//! # fn main() -> Result<()> {
//! let testpackage = testpackage! {
//!     "__init__.py" => "",
//!     "a.py" => "",
//!     "b.py" => "from django.db import models"
//! };
//!
//! let package_info = PackageInfo::build(testpackage.path())?;
//! let imports_info = ImportsInfo::build(package_info)?;
//!
//! let a = imports_info.package_info().get_item_by_pypath(&"testpackage.a".parse()?).unwrap().token();
//!
//! let contract = ForbiddenExternalImportContract::new(a, "django.db".parse()?);
//!
//! let result = contract.verify(&imports_info)?;
//! assert!(result.is_kept());
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Contract violated
//!
//! ```
//! # use anyhow::Result;
//! # use std::collections::HashSet;
//! # use pyimports::{testpackage};
//! # use pyimports::testutils::TestPackage;
//! use pyimports::package_info::PackageInfo;
//! use pyimports::imports_info::ImportsInfo;
//! use pyimports::contracts::{ImportsContract,ContractViolation,ForbiddenExternalImport};
//! use pyimports::contracts::forbidden_external::ForbiddenExternalImportContract;
//!
//! # fn main() -> Result<()> {
//! let testpackage = testpackage! {
//!     "__init__.py" => "",
//!     "a.py" => "from testpackage import b",
//!     "b.py" => "from django.db import models"
//! };
//!
//! let package_info = PackageInfo::build(testpackage.path())?;
//! let imports_info = ImportsInfo::build(package_info)?;
//!
//! let a = imports_info.package_info().get_item_by_pypath(&"testpackage.a".parse()?).unwrap().token();
//! let b = imports_info.package_info().get_item_by_pypath(&"testpackage.b".parse()?).unwrap().token();
//!
//! let contract = ForbiddenExternalImportContract::new(a, "django.db".parse()?);
//! let result = contract.verify(&imports_info)?;
//!
//! assert!(result.is_violated());
//! let expected_violations = [ContractViolation::ForbiddenExternalImport {
//!     forbidden_import: ForbiddenExternalImport::new(a, "django.db".parse()?, HashSet::new()),
//!     path: (vec![a, b], "django.db.models".parse()?),
//! }];
//! let violations = result.unwrap_violated();
//! assert_eq!(violations.len(), expected_violations.len());
//! for violation in violations.iter() {
//!     assert!(expected_violations.contains(violation));
//! }
//! # Ok(())
//! # }
//! ```

use crate::contracts::utils::{find_external_import_violations, ignore_imports};
use crate::contracts::{ContractVerificationResult, ForbiddenExternalImport, ImportsContract};
use crate::imports_info::ImportsInfo;
use crate::package_info::PackageItemToken;
use crate::pypath::Pypath;
use anyhow::Result;
use maplit::hashset;
use std::collections::HashSet;

/// A contract which forbids a certain external import.
/// See the [module-level documentation](./index.html) for more details.
#[derive(Debug, Clone)]
pub struct ForbiddenExternalImportContract {
    from: PackageItemToken,
    to: Pypath,
    except_via: HashSet<PackageItemToken>,
    ignored_internal_imports: Vec<(PackageItemToken, PackageItemToken)>,
    ignored_external_imports: Vec<(PackageItemToken, Pypath)>,
    ignore_typechecking_imports: bool,
}

impl ForbiddenExternalImportContract {
    /// Create a new [`ForbiddenExternalImportContract`].
    pub fn new(from: PackageItemToken, to: Pypath) -> Self {
        ForbiddenExternalImportContract {
            from,
            to,
            except_via: hashset! {},
            ignored_internal_imports: vec![],
            ignored_external_imports: vec![],
            ignore_typechecking_imports: false,
        }
    }

    /// Adds items by which the import path is allowed.
    pub fn with_except_via<T: Into<HashSet<PackageItemToken>>>(mut self, except_via: T) -> Self {
        let except_via = except_via.into();
        self.except_via = except_via;
        self
    }

    /// Ignore the passed internal imports when verifying the contract.
    pub fn with_ignored_internal_imports(
        mut self,
        imports: &[(PackageItemToken, PackageItemToken)],
    ) -> Self {
        self.ignored_internal_imports.extend(imports.to_vec());
        self
    }

    /// Ignore the passed external imports when verifying the contract.
    pub fn with_ignored_external_imports(mut self, imports: &[(PackageItemToken, Pypath)]) -> Self {
        self.ignored_external_imports.extend(imports.to_vec());
        self
    }

    /// Ignore typechecking imports when verifying the contract.
    pub fn with_typechecking_imports_ignored(mut self) -> Self {
        self.ignore_typechecking_imports = true;
        self
    }
}

impl ImportsContract for ForbiddenExternalImportContract {
    fn verify(&self, imports_info: &ImportsInfo) -> Result<ContractVerificationResult> {
        let imports_info = ignore_imports(
            imports_info,
            &self.ignored_internal_imports,
            &self.ignored_external_imports,
            self.ignore_typechecking_imports,
        )?;

        let forbidden_imports = [ForbiddenExternalImport::new(
            self.from,
            self.to.clone(),
            self.except_via.clone(),
        )];

        let violations = find_external_import_violations(&forbidden_imports, &imports_info)?;

        if violations.is_empty() {
            Ok(ContractVerificationResult::Kept)
        } else {
            Ok(ContractVerificationResult::Violated(violations))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{ContractViolation, ForbiddenExternalImport};
    use crate::package_info::PackageInfo;
    use crate::testpackage;
    use crate::testutils::TestPackage;
    use anyhow::Result;
    use maplit::hashset;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_forbidden_external_ok() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "a.py" => "",
            "b.py" => "from django.db import models"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let a = imports_info.package_info()._item("testpackage.a");

        let contract = ForbiddenExternalImportContract::new(a, "django.db".parse()?);

        let result = contract.verify(&imports_info)?;
        assert!(result.is_kept());

        Ok(())
    }

    #[test]
    fn test_forbidden_external_violated() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "a.py" => "from testpackage import b",
            "b.py" => "from django.db import models"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let a = imports_info.package_info()._item("testpackage.a");
        let b = imports_info.package_info()._item("testpackage.b");

        let contract = ForbiddenExternalImportContract::new(a, "django.db".parse()?);

        let result = contract.verify(&imports_info)?;
        assert!(result.is_violated());
        let expected_violations = [ContractViolation::ForbiddenExternalImport {
            forbidden_import: ForbiddenExternalImport::new(a, "django.db".parse()?, hashset! {}),
            path: (vec![a, b], "django.db.models".parse()?),
        }];
        let violations = result.unwrap_violated();
        assert_eq!(violations.len(), expected_violations.len());
        for violation in violations.iter() {
            assert!(expected_violations.contains(violation));
        }

        Ok(())
    }
}
