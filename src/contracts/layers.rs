//! The `layers` module provides a [`LayeredArchitectureContract`], which enforces a layered architecture.
//!
//! A layered architecture involves a set of layers, with rules for imports between layers:
//!
//! - Lower layers may not import higher layers.
//! - Siblings within a layer may be marked as independent, in which case they may
//!   not import each other.
//! - Higher layers may import lower layers. By default higher layers may only import from the
//!   immediately below layer. This restriction may be lifted via [`LayeredArchitectureContract::with_deep_imports_allowed`].
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
//! use pyimports::contracts::layers::{LayeredArchitectureContract,Layer};
//!
//! # fn main() -> Result<()> {
//! let testpackage = testpackage! {
//!     "data.py" => "",
//!     "domain.py" => "import testpackage.data",
//!     "application.py" => "import testpackage.domain",
//!     "interfaces.py" => "import testpackage.application"
//! };
//!
//! let package_info = PackageInfo::build(testpackage.path())?;
//! let imports_info = ImportsInfo::build(package_info)?;
//!
//! let data = imports_info.package_info().get_item_by_pypath("testpackage.data")?.unwrap().token();
//! let domain = imports_info.package_info().get_item_by_pypath("testpackage.domain")?.unwrap().token();
//! let application = imports_info.package_info().get_item_by_pypath("testpackage.application")?.unwrap().token();
//! let interfaces = imports_info.package_info().get_item_by_pypath("testpackage.interfaces")?.unwrap().token();
//!
//! let contract = LayeredArchitectureContract::new(&[
//!     Layer::new([data], true),
//!     Layer::new([domain], true),
//!     Layer::new([application], true),
//!     Layer::new([interfaces], true),
//! ]);
//!
//! let result = contract.verify(&imports_info)?;
//!
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
//! use pyimports::contracts::{ImportsContract,ContractViolation,ForbiddenImport};
//! use pyimports::contracts::layers::{LayeredArchitectureContract,Layer};
//!
//! # fn main() -> Result<()> {
//! let testpackage = testpackage! {
//!     "data.py" => "",
//!     "domain.py" => "import testpackage.data",
//!     "application.py" => "
//! import testpackage.domain
//! import testpackage.interfaces",
//!     "interfaces.py" => "import testpackage.application"
//! };
//!
//! let package_info = PackageInfo::build(testpackage.path())?;
//! let imports_info = ImportsInfo::build(package_info)?;
//!
//! let data = imports_info.package_info().get_item_by_pypath("testpackage.data")?.unwrap().token();
//! let domain = imports_info.package_info().get_item_by_pypath("testpackage.domain")?.unwrap().token();
//! let application = imports_info.package_info().get_item_by_pypath("testpackage.application")?.unwrap().token();
//! let interfaces = imports_info.package_info().get_item_by_pypath("testpackage.interfaces")?.unwrap().token();
//!
//! let contract = LayeredArchitectureContract::new(&[
//!     Layer::new([data], true),
//!     Layer::new([domain], true),
//!     Layer::new([application], true),
//!     Layer::new([interfaces], true),
//! ]);
//!
//! let result = contract.verify(&imports_info)?;
//!
//! assert!(result.is_violated());
//! let violations = result.unwrap_violated();
//! assert_eq!(
//!     violations,
//!     vec![
//!         ContractViolation::ForbiddenImport {
//!             forbidden_import: ForbiddenImport::new(application, interfaces, HashSet::new()),
//!             path: vec![application, interfaces],
//!         },
//!     ]
//! );
//! # Ok(())
//! # }
//! ```

use crate::contracts::utils::find_violations;
use crate::contracts::{ContractVerificationResult, ForbiddenImport, ImportsContract};
use crate::imports_info::ImportsInfo;
use crate::package_info::PackageItemToken;
use anyhow::Result;
use itertools::Itertools;
use maplit::hashset;
use std::collections::HashSet;

/// A contract used to enforce a layered architecture.
/// See the [module-level documentation](./index.html) for more details.
#[derive(Debug, Clone)]
pub struct LayeredArchitectureContract {
    layers: Vec<Layer>,
    ignored_imports: Vec<(PackageItemToken, PackageItemToken)>,
    ignore_typechecking_imports: bool,
    allow_deep_imports: bool,
}

impl LayeredArchitectureContract {
    /// Create a new [`LayeredArchitectureContract`].
    /// Layers should be listed from lowest to highest.
    pub fn new(layers: &[Layer]) -> Self {
        LayeredArchitectureContract {
            layers: layers.to_vec(),
            ignored_imports: vec![],
            ignore_typechecking_imports: false,
            allow_deep_imports: false,
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

    /// Allow deep imports.
    ///
    /// By default higher layers may only import the immediately below layer.
    /// `allow_deep_imports` lifts this restriction.   
    pub fn with_deep_imports_allowed(mut self) -> Self {
        self.allow_deep_imports = true;
        self
    }
}

impl ImportsContract for LayeredArchitectureContract {
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

        let forbidden_imports = get_forbidden_imports(&self.layers, self.allow_deep_imports);

        let violations = find_violations(forbidden_imports, &imports_info)?;

        if violations.is_empty() {
            Ok(ContractVerificationResult::Kept)
        } else {
            Ok(ContractVerificationResult::Violated(violations))
        }
    }
}

/// A layer within a layered architecture.
/// See the [module-level documentation](./index.html) for more details.
#[derive(Debug, Clone)]
pub struct Layer {
    siblings: HashSet<PackageItemToken>,
    siblings_independent: bool,
}

impl Layer {
    /// Creates a new layer.
    pub fn new<T: IntoIterator<Item = PackageItemToken>>(
        siblings: T,
        siblings_independent: bool,
    ) -> Self {
        Layer {
            siblings: siblings.into_iter().collect(),
            siblings_independent,
        }
    }
}

fn get_forbidden_imports(layers: &[Layer], allow_deep_imports: bool) -> Vec<ForbiddenImport> {
    let mut forbidden_imports = Vec::new();

    for (idx, layer) in layers.iter().enumerate() {
        // Lower layers should not import higher layers.
        for higher_layer in layers[idx + 1..].iter() {
            for layer_sibling in layer.siblings.iter() {
                for higher_layer_sibling in higher_layer.siblings.iter() {
                    forbidden_imports.push(ForbiddenImport::new(
                        *layer_sibling,
                        *higher_layer_sibling,
                        hashset! {},
                    ));
                }
            }
        }

        if !allow_deep_imports {
            // Higher layers should not import lower layers, except via the layer immediately below.
            if idx >= 2 {
                let directly_lower_layer = &layers[idx - 1];
                for lower_layer in layers[..idx - 1].iter() {
                    for layer_sibling in layer.siblings.iter() {
                        for lower_layer_sibling in lower_layer.siblings.iter() {
                            forbidden_imports.push(ForbiddenImport::new(
                                *layer_sibling,
                                *lower_layer_sibling,
                                directly_lower_layer.siblings.clone(),
                            ));
                        }
                    }
                }
            }
        }

        // Independent siblings should not import each other.
        if layer.siblings_independent {
            for permutation in layer.siblings.iter().permutations(2) {
                forbidden_imports.push(ForbiddenImport::new(
                    *permutation[0],
                    *permutation[1],
                    hashset! {},
                ));
            }
        }
    }

    forbidden_imports
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::ContractViolation;
    use crate::package_info::{PackageInfo, PackageToken};
    use crate::testpackage;
    use crate::testutils::TestPackage;
    use anyhow::Result;
    use maplit::hashset;
    use pretty_assertions::assert_eq;
    use slotmap::SlotMap;

    #[test]
    fn test_get_forbidden_imports() -> Result<()> {
        let mut sm: SlotMap<PackageToken, String> = SlotMap::with_key();
        let data: PackageItemToken = sm.insert("data".into()).into();
        let domain1: PackageItemToken = sm.insert("domain1".into()).into();
        let domain2: PackageItemToken = sm.insert("domain2".into()).into();
        let application1: PackageItemToken = sm.insert("application1".into()).into();
        let application2: PackageItemToken = sm.insert("application2".into()).into();
        let interfaces: PackageItemToken = sm.insert("interfaces".into()).into();

        let layers = vec![
            Layer::new([data], true),
            Layer::new([domain1, domain2], true),
            Layer::new([application1, application2], false),
            Layer::new([interfaces], true),
        ];

        let forbidden_imports = get_forbidden_imports(&layers, false);

        let expected = vec![
            // data may not import domain, application or interfaces
            ForbiddenImport::new(data, domain1, hashset! {}),
            ForbiddenImport::new(data, domain2, hashset! {}),
            ForbiddenImport::new(data, application1, hashset! {}),
            ForbiddenImport::new(data, application2, hashset! {}),
            ForbiddenImport::new(data, interfaces, hashset! {}),
            // domain may not import application or interfaces
            // (domain may import data)
            ForbiddenImport::new(domain1, application1, hashset! {}),
            ForbiddenImport::new(domain1, application2, hashset! {}),
            ForbiddenImport::new(domain1, interfaces, hashset! {}),
            ForbiddenImport::new(domain2, application1, hashset! {}),
            ForbiddenImport::new(domain2, application2, hashset! {}),
            ForbiddenImport::new(domain2, interfaces, hashset! {}),
            // domain1 and domain2 are independent siblings
            ForbiddenImport::new(domain1, domain2, hashset! {}),
            ForbiddenImport::new(domain2, domain1, hashset! {}),
            // application may not import interfaces
            // application may not import data, except via domain
            // (application may import domain)
            ForbiddenImport::new(application1, interfaces, hashset! {}),
            ForbiddenImport::new(application1, data, hashset! {domain1, domain2}),
            ForbiddenImport::new(application2, interfaces, hashset! {}),
            ForbiddenImport::new(application2, data, hashset! {domain1, domain2}),
            // interfaces may not import data or domain, except via application
            // (application may import application)
            ForbiddenImport::new(interfaces, data, hashset! {application1, application2}),
            ForbiddenImport::new(interfaces, domain1, hashset! {application1, application2}),
            ForbiddenImport::new(interfaces, domain2, hashset! {application1, application2}),
        ];

        assert_eq!(forbidden_imports.len(), expected.len(),);
        for forbidden_import in forbidden_imports.iter() {
            assert!(expected.contains(forbidden_import));
        }

        Ok(())
    }

    #[test]
    fn test_layered_architecture_contract_ok() -> Result<()> {
        let testpackage = testpackage! {
                "__init__.py" => "",
                "data.py" => "",
                "domain.py" => "
import testpackage.data
",
        "application.py" => "
import testpackage.domain
",
        "interfaces.py" => "
import testpackage.application
"
            };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let data = imports_info.package_info()._item("testpackage.data");
        let domain = imports_info.package_info()._item("testpackage.domain");
        let application = imports_info.package_info()._item("testpackage.application");
        let interfaces = imports_info.package_info()._item("testpackage.interfaces");

        let contract = LayeredArchitectureContract::new(&[
            Layer::new([data], true),
            Layer::new([domain], true),
            Layer::new([application], true),
            Layer::new([interfaces], true),
        ]);

        let result = contract.verify(&imports_info)?;
        assert!(result.is_kept());

        Ok(())
    }

    #[test]
    fn test_layered_architecture_contract_violated() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "data.py" => "",
            "domain.py" => "
import testpackage.data
",
            "application.py" => "
import testpackage.domain
import testpackage.interfaces
",
            "interfaces.py" => "
import testpackage.application
import testpackage.data
"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let data = imports_info.package_info()._item("testpackage.data");
        let domain = imports_info.package_info()._item("testpackage.domain");
        let application = imports_info.package_info()._item("testpackage.application");
        let interfaces = imports_info.package_info()._item("testpackage.interfaces");

        let contract = LayeredArchitectureContract::new(&[
            Layer::new([data], true),
            Layer::new([domain], true),
            Layer::new([application], true),
            Layer::new([interfaces], true),
        ]);

        let result = contract.verify(&imports_info)?;

        assert!(result.is_violated());
        let expected_violations = vec![
            ContractViolation::ForbiddenImport {
                forbidden_import: ForbiddenImport::new(application, interfaces, hashset! {}),
                path: vec![application, interfaces],
            },
            ContractViolation::ForbiddenImport {
                forbidden_import: ForbiddenImport::new(interfaces, data, hashset! {application}),
                path: vec![interfaces, data],
            },
            ContractViolation::ForbiddenImport {
                forbidden_import: ForbiddenImport::new(application, data, hashset! {domain}),
                path: vec![application, interfaces, data],
            },
        ];
        let violations = result.unwrap_violated();
        assert_eq!(violations.len(), expected_violations.len());
        for violation in violations.iter() {
            assert!(expected_violations.contains(violation));
        }

        Ok(())
    }

    #[test]
    fn test_layered_architecture_contract_violated_ignored_imports() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "data.py" => "",
            "domain.py" => "
import testpackage.data
",
            "application.py" => "
import testpackage.domain
import testpackage.interfaces
",
            "interfaces.py" => "
import testpackage.application
import testpackage.data
"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let data = imports_info.package_info()._item("testpackage.data");
        let domain = imports_info.package_info()._item("testpackage.domain");
        let application = imports_info.package_info()._item("testpackage.application");
        let interfaces = imports_info.package_info()._item("testpackage.interfaces");

        let contract = LayeredArchitectureContract::new(&[
            Layer::new([data], true),
            Layer::new([domain], true),
            Layer::new([application], true),
            Layer::new([interfaces], true),
        ])
        .with_ignored_imports(&[(interfaces, data)]);

        let result = contract.verify(&imports_info)?;

        let expected_violations = [ContractViolation::ForbiddenImport {
            forbidden_import: ForbiddenImport::new(application, interfaces, hashset! {}),
            path: vec![application, interfaces],
        }];
        let violations = result.unwrap_violated();
        assert_eq!(violations.len(), expected_violations.len());
        for violation in violations.iter() {
            assert!(expected_violations.contains(violation));
        }

        Ok(())
    }

    #[test]
    fn test_layered_architecture_contract_allowing_deep_imports() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "data.py" => "",
            "domain.py" => "
import testpackage.data
",
            "application.py" => "
import testpackage.domain
import testpackage.data  # A deep import
",
            "interfaces.py" => "
import testpackage.application
"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let data = imports_info.package_info()._item("testpackage.data");
        let domain = imports_info.package_info()._item("testpackage.domain");
        let application = imports_info.package_info()._item("testpackage.application");
        let interfaces = imports_info.package_info()._item("testpackage.interfaces");

        // Sanity check
        let contract = LayeredArchitectureContract::new(&[
            Layer::new([data], true),
            Layer::new([domain], true),
            Layer::new([application], true),
            Layer::new([interfaces], true),
        ]);
        let result = contract.verify(&imports_info)?;
        let expected_violations = [ContractViolation::ForbiddenImport {
            forbidden_import: ForbiddenImport::new(application, data, hashset! {domain}),
            path: vec![application, data],
        }];
        let violations = result.unwrap_violated();
        assert_eq!(violations.len(), expected_violations.len());
        for violation in violations.iter() {
            assert!(expected_violations.contains(violation));
        }

        // Allowing deep imports
        let contract = LayeredArchitectureContract::new(&[
            Layer::new([data], true),
            Layer::new([domain], true),
            Layer::new([application], true),
            Layer::new([interfaces], true),
        ])
        .with_deep_imports_allowed();
        let result = contract.verify(&imports_info)?;
        assert!(result.is_kept());

        Ok(())
    }
}
