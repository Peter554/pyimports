//! The `forbidden_internal` module provides a [`ForbiddenInternalImportContract`], which forbids a certain internal import.
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
//! use pyimports::contracts::forbidden_internal::ForbiddenInternalImportContract;
//!
//! # fn main() -> Result<()> {
//! let testpackage = testpackage! {
//!     "__init__.py" => "",
//!     "a.py" => "import testpackage.c",
//!     "b.py" => "",
//!     "c.py" => ""
//! };
//!
//! let package_info = PackageInfo::build(testpackage.path())?;
//! let imports_info = ImportsInfo::build(package_info)?;
//!
//! let a = imports_info.package_info().get_item_by_pypath(&"testpackage.a".parse()?).unwrap().token();
//! let b = imports_info.package_info().get_item_by_pypath(&"testpackage.b".parse()?).unwrap().token();
//!
//! let contract = ForbiddenInternalImportContract::new(a, b);
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
//! # use maplit::hashset;
//! # use std::collections::HashSet;
//! # use pyimports::{testpackage};
//! # use pyimports::testutils::TestPackage;
//! use pyimports::package_info::PackageInfo;
//! use pyimports::imports_info::ImportsInfo;
//! use pyimports::contracts::{ImportsContract,ContractViolation,ForbiddenInternalImport};
//! use pyimports::contracts::forbidden_internal::ForbiddenInternalImportContract;
//!
//! # fn main() -> Result<()> {
//! let testpackage = testpackage! {
//!     "__init__.py" => "",
//!     "a.py" => "import testpackage.c",
//!     "b.py" => "",
//!     "c.py" => "import testpackage.b"
//! };
//!
//! let package_info = PackageInfo::build(testpackage.path())?;
//! let imports_info = ImportsInfo::build(package_info)?;
//!
//! let a = imports_info.package_info().get_item_by_pypath(&"testpackage.a".parse()?).unwrap().token();
//! let b = imports_info.package_info().get_item_by_pypath(&"testpackage.b".parse()?).unwrap().token();
//! let c = imports_info.package_info().get_item_by_pypath(&"testpackage.c".parse()?).unwrap().token();
//!
//! let contract = ForbiddenInternalImportContract::new(a, b);
//!
//! let result = contract.verify(&imports_info)?;
//! assert!(result.is_violated());
//! let expected_violations = [ContractViolation::ForbiddenInternalImport {
//!     forbidden_import: ForbiddenInternalImport::new(a, b, hashset! {}),
//!     path: vec![a, c, b],
//! }];
//! let violations = result.unwrap_violated();
//! assert_eq!(violations.len(), expected_violations.len());
//! for violation in violations.iter() {
//!     assert!(expected_violations.contains(violation));
//! }
//! # Ok(())
//! # }
//! ```

use crate::contracts::utils::{find_violations, ignore_imports};
use crate::contracts::{ContractVerificationResult, ForbiddenInternalImport, ImportsContract};
use crate::imports_info::ImportsInfo;
use crate::package_info::PackageItemToken;
use anyhow::Result;
use maplit::hashset;
use std::collections::HashSet;

/// A contract which forbids a certain internal import.
/// See the [module-level documentation](./index.html) for more details.
#[derive(Debug, Clone)]
pub struct ForbiddenInternalImportContract {
    from: PackageItemToken,
    to: PackageItemToken,
    except_via: HashSet<PackageItemToken>,
    ignored_imports: Vec<(PackageItemToken, PackageItemToken)>,
    ignore_typechecking_imports: bool,
}

impl ForbiddenInternalImportContract {
    /// Create a new [`ForbiddenInternalImportContract`].
    pub fn new(from: PackageItemToken, to: PackageItemToken) -> Self {
        ForbiddenInternalImportContract {
            from,
            to,
            except_via: hashset! {},
            ignored_imports: vec![],
            ignore_typechecking_imports: false,
        }
    }

    /// Adds items by which the import path is allowed.
    pub fn with_except_via<T: Into<HashSet<PackageItemToken>>>(mut self, except_via: T) -> Self {
        let except_via = except_via.into();
        self.except_via = except_via;
        self
    }

    /// Ignore the passed imports when verifying the contract.
    pub fn with_ignored_imports(
        mut self,
        imports: &[(PackageItemToken, PackageItemToken)],
    ) -> Self {
        self.ignored_imports.extend(imports.to_vec());
        self
    }

    /// Ignore typechecking imports when verifying the contract.
    pub fn with_typechecking_imports_ignored(mut self) -> Self {
        self.ignore_typechecking_imports = true;
        self
    }
}

impl ImportsContract for ForbiddenInternalImportContract {
    fn verify(&self, imports_info: &ImportsInfo) -> Result<ContractVerificationResult> {
        let imports_info = ignore_imports(
            imports_info,
            &self.ignored_imports,
            &[],
            self.ignore_typechecking_imports,
        )?;

        let forbidden_imports = [ForbiddenInternalImport::new(
            self.from,
            self.to,
            self.except_via.clone(),
        )];

        let violations = find_violations(&forbidden_imports, &imports_info)?;

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
    use crate::contracts::ContractViolation;
    use crate::package_info::PackageInfo;
    use crate::testpackage;
    use crate::testutils::TestPackage;
    use anyhow::Result;
    use maplit::hashset;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_forbidden_internal_ok() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "a.py" => "import testpackage.c",
            "b.py" => "",
            "c.py" => ""
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let a = imports_info.package_info()._item("testpackage.a");
        let b = imports_info.package_info()._item("testpackage.b");

        let contract = ForbiddenInternalImportContract::new(a, b);

        let result = contract.verify(&imports_info)?;
        assert!(result.is_kept());

        Ok(())
    }

    #[test]
    fn test_forbidden_internal_violated() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "a.py" => "import testpackage.c",
            "b.py" => "",
            "c.py" => "import testpackage.b"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let a = imports_info.package_info()._item("testpackage.a");
        let b = imports_info.package_info()._item("testpackage.b");
        let c = imports_info.package_info()._item("testpackage.c");

        let contract = ForbiddenInternalImportContract::new(a, b);

        let result = contract.verify(&imports_info)?;
        assert!(result.is_violated());
        let expected_violations = [ContractViolation::ForbiddenInternalImport {
            forbidden_import: ForbiddenInternalImport::new(a, b, hashset! {}),
            path: vec![a, c, b],
        }];
        let violations = result.unwrap_violated();
        assert_eq!(violations.len(), expected_violations.len());
        for violation in violations.iter() {
            assert!(expected_violations.contains(violation));
        }

        Ok(())
    }

    #[test]
    fn test_forbidden_internal_except_via() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "a.py" => "import testpackage.c",
            "b.py" => "",
            "c.py" => "import testpackage.b"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let a = imports_info.package_info()._item("testpackage.a");
        let b = imports_info.package_info()._item("testpackage.b");
        let c = imports_info.package_info()._item("testpackage.c");

        let contract = ForbiddenInternalImportContract::new(a, b);
        let result = contract.verify(&imports_info)?;
        assert!(result.is_violated());

        let contract = ForbiddenInternalImportContract::new(a, b).with_except_via(c);
        let result = contract.verify(&imports_info)?;
        assert!(result.is_kept());

        Ok(())
    }
}
