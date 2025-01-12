//! The `independent` module provides a [`IndependentItemsContract`], which ensures that all items are independent.
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
//! use pyimports::contracts::independent::IndependentItemsContract;
//!
//! # fn main() -> Result<()> {
//! let testpackage = testpackage! {
//!     "__init__.py" => "",
//!     "a.py" => "import testpackage.c",
//!     "b.py" => "import testpackage.d",
//!     "c.py" => "",
//!     "d.py" => ""
//! };
//!
//! let package_info = PackageInfo::build(testpackage.path())?;
//! let imports_info = ImportsInfo::build(package_info)?;
//!
//! let a = imports_info.package_info().get_item_by_pypath(&"testpackage.a".parse()?).unwrap().token();
//! let b = imports_info.package_info().get_item_by_pypath(&"testpackage.b".parse()?).unwrap().token();
//!
//! let contract = IndependentItemsContract::new(&[a, b]);
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
//! use pyimports::contracts::independent::IndependentItemsContract;
//!
//! # fn main() -> Result<()> {
//! let testpackage = testpackage! {
//!     "__init__.py" => "",
//!     "a.py" => "import testpackage.c",
//!     "b.py" => "import testpackage.d",
//!     "c.py" => "import testpackage.b",
//!     "d.py" => "import testpackage.a"
//! };
//!
//! let package_info = PackageInfo::build(testpackage.path())?;
//! let imports_info = ImportsInfo::build(package_info)?;
//!
//! let a = imports_info.package_info().get_item_by_pypath(&"testpackage.a".parse()?).unwrap().token();
//! let b = imports_info.package_info().get_item_by_pypath(&"testpackage.b".parse()?).unwrap().token();
//! let c = imports_info.package_info().get_item_by_pypath(&"testpackage.c".parse()?).unwrap().token();
//! let d = imports_info.package_info().get_item_by_pypath(&"testpackage.d".parse()?).unwrap().token();
//!
//! let contract = IndependentItemsContract::new(&[a, b]);
//!
//! let result = contract.verify(&imports_info)?;
//! assert!(result.is_violated());
//! let expected_violations = [
//!     ContractViolation::ForbiddenInternalImport {
//!         forbidden_import: ForbiddenInternalImport::new(a, b, hashset! {}),
//!         path: vec![a, c, b],
//!     },
//!     ContractViolation::ForbiddenInternalImport {
//!         forbidden_import: ForbiddenInternalImport::new(b, a, hashset! {}),
//!         path: vec![b, d, a],
//!     },
//! ];
//! let violations = result.unwrap_violated();
//! assert_eq!(violations.len(), expected_violations.len());
//! for violation in violations.iter() {
//!     assert!(expected_violations.contains(violation));
//! }
//! # Ok(())
//! # }
//! ```

use crate::contracts::utils::find_violations;
use crate::contracts::{ContractVerificationResult, ForbiddenInternalImport, ImportsContract};
use crate::imports_info::ImportsInfo;
use crate::package_info::PackageItemToken;
use anyhow::Result;
use itertools::Itertools;
use maplit::hashset;
use std::collections::HashSet;

/// A contract which ensures that all items are independent.
/// See the [module-level documentation](./index.html) for more details.
#[derive(Debug, Clone)]
pub struct IndependentItemsContract {
    items: HashSet<PackageItemToken>,
    ignored_imports: Vec<(PackageItemToken, PackageItemToken)>,
    ignore_typechecking_imports: bool,
}

impl IndependentItemsContract {
    /// Create a new [`IndependentItemsContract`].
    pub fn new(items: &[PackageItemToken]) -> Self {
        IndependentItemsContract {
            items: items.iter().cloned().collect(),
            ignored_imports: vec![],
            ignore_typechecking_imports: false,
        }
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

impl ImportsContract for IndependentItemsContract {
    fn verify(&self, imports_info: &ImportsInfo) -> Result<ContractVerificationResult> {
        // Assumption: It's best/reasonable to clone here and remove the ignored imports from the graph.
        // An alternative could be to ignore the imports dynamically via a new field on `InternalImportsPathQuery`.
        let imports_info = {
            let mut imports_info = imports_info.clone();
            if !self.ignored_imports.is_empty() {
                imports_info.remove_imports(self.ignored_imports.clone(), [])?;
            }
            if self.ignore_typechecking_imports {
                imports_info.remove_typechecking_imports()?;
            }
            imports_info
        };

        let forbidden_imports = self
            .items
            .iter()
            .permutations(2)
            .map(|permutation| {
                ForbiddenInternalImport::new(*permutation[0], *permutation[1], hashset! {})
            })
            .collect::<Vec<_>>();

        let violations = find_violations(forbidden_imports, &imports_info)?;

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
    fn test_independent_items_ok() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "a.py" => "import testpackage.c",
            "b.py" => "import testpackage.d",
            "c.py" => "",
            "d.py" => ""
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let a = imports_info.package_info()._item("testpackage.a");
        let b = imports_info.package_info()._item("testpackage.b");

        let contract = IndependentItemsContract::new(&[a, b]);

        let result = contract.verify(&imports_info)?;
        assert!(result.is_kept());

        Ok(())
    }

    #[test]
    fn test_independent_items_violated() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "a.py" => "import testpackage.c",
            "b.py" => "import testpackage.d",
            "c.py" => "import testpackage.b",
            "d.py" => "import testpackage.a"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let a = imports_info.package_info()._item("testpackage.a");
        let b = imports_info.package_info()._item("testpackage.b");
        let c = imports_info.package_info()._item("testpackage.c");
        let d = imports_info.package_info()._item("testpackage.d");

        let contract = IndependentItemsContract::new(&[a, b]);

        let result = contract.verify(&imports_info)?;
        assert!(result.is_violated());
        let expected_violations = [
            ContractViolation::ForbiddenInternalImport {
                forbidden_import: ForbiddenInternalImport::new(a, b, hashset! {}),
                path: vec![a, c, b],
            },
            ContractViolation::ForbiddenInternalImport {
                forbidden_import: ForbiddenInternalImport::new(b, a, hashset! {}),
                path: vec![b, d, a],
            },
        ];
        let violations = result.unwrap_violated();
        assert_eq!(violations.len(), expected_violations.len());
        for violation in violations.iter() {
            assert!(expected_violations.contains(violation));
        }

        Ok(())
    }
}
