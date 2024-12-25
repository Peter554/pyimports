mod ast_visit;
pub mod one_file;

use anyhow::Result;
use one_file::RawImport;
use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::package_discovery::{PackageInfo, PackageItemToken};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportMetadata {
    line_number: usize,
    is_typechecking: bool,
}

#[derive(Debug, Clone)]
pub struct ImportsInfo {
    package_info: PackageInfo,
    //
    internal_imports: HashMap<PackageItemToken, HashSet<PackageItemToken>>,
    reverse_internal_imports: HashMap<PackageItemToken, HashSet<PackageItemToken>>,
    internal_imports_metadata: HashMap<(PackageItemToken, PackageItemToken), ImportMetadata>,
}

impl ImportsInfo {
    pub fn build(package_info: PackageInfo) -> Result<Self> {
        let all_raw_imports = get_all_raw_imports(&package_info)?;

        let mut imports_info = ImportsInfo {
            package_info: package_info.clone(),
            internal_imports: HashMap::new(),
            reverse_internal_imports: HashMap::new(),
            internal_imports_metadata: HashMap::new(),
        };

        imports_info.initialise_maps()?;

        // By definition, packages import their init modules.
        for package in package_info
            .get_all_items()
            .filter_map(PackageInfo::filter_packages)
        {
            if let Some(init_module) = package.init_module {
                imports_info.add_internal_import(package.token.into(), init_module.into(), None)?;
            }
        }

        for (item, raw_imports) in all_raw_imports {
            for raw_import in raw_imports {
                let metadata = ImportMetadata {
                    line_number: raw_import.line_number,
                    is_typechecking: raw_import.is_typechecking,
                };

                // Try to find an internal import.
                let internal_item = {
                    if let Some(item) = package_info
                        .get_item_by_pypath(&raw_import.pypath)
                        .map(|item: crate::package_discovery::PackageItem<'_>| item.token())
                    {
                        // An imported module.
                        Some(item)
                    } else {
                        // An imported module member.
                        // e.g. from testpackage.foo import FooClass
                        // The pypath is testpackage.foo.FooClass, so we need to strip the final part.
                        package_info
                            .get_item_by_pypath(&strip_final_part(&raw_import.pypath))
                            .map(|item: crate::package_discovery::PackageItem<'_>| item.token())
                    }
                };

                match internal_item {
                    Some(internal_item) => {
                        imports_info.add_internal_import(item, internal_item, Some(metadata))?;
                    }
                    None => {
                        // Do not consider external imports, for now.
                        // TODO: External imports?
                    }
                }
            }
        }

        Ok(imports_info)
    }

    pub fn package_info(&self) -> &PackageInfo {
        &self.package_info
    }

    pub fn exclude_imports(
        &self,
        imports: impl IntoIterator<Item = (PackageItemToken, PackageItemToken)>,
    ) -> Result<Self> {
        let mut imports_info = self.clone();
        for (from, to) in imports {
            imports_info.remove_internal_import(from, to)?;
        }
        Ok(imports_info)
    }

    pub fn exclude_typechecking_imports(&self) -> Result<Self> {
        let imports = self
            .internal_imports_metadata
            .iter()
            .filter_map(|((from, to), metadata)| {
                if metadata.is_typechecking {
                    Some((*from, *to))
                } else {
                    None
                }
            });
        self.exclude_imports(imports)
    }

    fn initialise_maps(&mut self) -> Result<()> {
        for item in self.package_info.get_all_items() {
            self.internal_imports.entry(item.token()).or_default();
            self.reverse_internal_imports
                .entry(item.token())
                .or_default();
        }
        Ok(())
    }

    fn add_internal_import(
        &mut self,
        from: PackageItemToken,
        to: PackageItemToken,
        metadata: Option<ImportMetadata>,
    ) -> Result<()> {
        self.internal_imports.entry(from).or_default().insert(to);
        self.reverse_internal_imports
            .entry(to)
            .or_default()
            .insert(from);
        if let Some(metadata) = metadata {
            self.internal_imports_metadata.insert((from, to), metadata);
        }
        Ok(())
    }

    fn remove_internal_import(
        &mut self,
        from: PackageItemToken,
        to: PackageItemToken,
    ) -> Result<()> {
        self.internal_imports.entry(from).or_default().remove(&to);
        self.reverse_internal_imports
            .entry(to)
            .or_default()
            .remove(&from);
        self.internal_imports_metadata.remove(&(from, to));
        Ok(())
    }
}

fn get_all_raw_imports(
    package_info: &PackageInfo,
) -> Result<HashMap<PackageItemToken, Vec<RawImport>>> {
    let all_raw_imports = package_info
        .get_all_items()
        .filter_map(PackageInfo::filter_modules)
        .par_bridge()
        .try_fold(
            HashMap::new,
            |mut hm: HashMap<PackageItemToken, Vec<RawImport>>, module| -> Result<_> {
                // Parse the raw imports.
                let raw_imports = one_file::discover_imports(&module.path)?;
                // Resolve any relative imports.
                let raw_imports = one_file::resolve_relative_imports(
                    &module.path,
                    raw_imports,
                    &package_info.get_root().path,
                )?;

                hm.entry(module.token.into())
                    .or_default()
                    .extend(raw_imports);

                Ok(hm)
            },
        )
        .try_reduce(HashMap::new, |mut hm, h| {
            for (k, v) in h {
                hm.entry(k).or_default().extend(v);
            }
            Ok(hm)
        })?;
    Ok(all_raw_imports)
}

fn strip_final_part(pypath: &str) -> String {
    let mut o = pypath.split(".").collect::<Vec<_>>();
    o.pop();
    o.join(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutils::{testpackage, TestPackage};
    use maplit::{hashmap, hashset};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_build() -> Result<()> {
        let test_package = testpackage! {
            "__init__.py" => "
from testpackage import a
from testpackage import b
",

            "a.py" => "
from testpackage.b import HELLO
",

            "b.py" => "
from django.db import models
"
        };

        let package_info = PackageInfo::build(test_package.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let root_package = imports_info
            .package_info
            .get_item_by_pypath("testpackage")
            .unwrap()
            .token();
        let root_package_init = imports_info
            .package_info
            .get_item_by_pypath("testpackage.__init__")
            .unwrap()
            .token();
        let a = imports_info
            .package_info
            .get_item_by_pypath("testpackage.a")
            .unwrap()
            .token();
        let b = imports_info
            .package_info
            .get_item_by_pypath("testpackage.b")
            .unwrap()
            .token();

        assert_eq!(
            imports_info.internal_imports,
            hashmap! {
                root_package => hashset! {root_package_init},
                root_package_init => hashset! {a, b},
                a => hashset! {b},
                b => hashset!{},
            }
        );

        assert_eq!(
            imports_info.reverse_internal_imports,
            hashmap! {
                root_package => hashset!{},
                root_package_init => hashset! {root_package},
                a => hashset! {root_package_init},
                b => hashset! {root_package_init, a},
            }
        );

        assert_eq!(
            imports_info.internal_imports_metadata,
            hashmap! {
                (root_package_init, a) => ImportMetadata{
                    line_number: 2,
                    is_typechecking: false,
                },
                (root_package_init, b) => ImportMetadata{
                    line_number: 3,
                    is_typechecking: false,
                },
                (a, b) => ImportMetadata{
                    line_number: 2,
                    is_typechecking: false,
                }
            }
        );

        Ok(())
    }

    #[test]
    fn test_exclude_imports() -> Result<()> {
        let test_package = testpackage! {
            "__init__.py" => "
import testpackage.a
from testpackage import b
",
            "a.py" => "",
            "b.py" => ""
        };

        let package_info = PackageInfo::build(test_package.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let root_package = imports_info
            .package_info
            .get_item_by_pypath("testpackage")
            .unwrap()
            .token();
        let root_package_init = imports_info
            .package_info
            .get_item_by_pypath("testpackage.__init__")
            .unwrap()
            .token();
        let a = imports_info
            .package_info
            .get_item_by_pypath("testpackage.a")
            .unwrap()
            .token();
        let b = imports_info
            .package_info
            .get_item_by_pypath("testpackage.b")
            .unwrap()
            .token();

        assert_eq!(
            imports_info.internal_imports,
            hashmap! {
                root_package => hashset! {root_package_init},
                root_package_init => hashset! {a, b},
                a => hashset! {},
                b => hashset!{},
            }
        );

        assert_eq!(
            imports_info.reverse_internal_imports,
            hashmap! {
                root_package => hashset!{},
                root_package_init => hashset! {root_package},
                a => hashset! {root_package_init},
                b => hashset! {root_package_init},
            }
        );

        assert_eq!(
            imports_info.internal_imports_metadata,
            hashmap! {
                (root_package_init, a) => ImportMetadata{
                    line_number: 2,
                    is_typechecking: false,
                },
                (root_package_init, b) => ImportMetadata{
                    line_number: 3,
                    is_typechecking: false,
                },
            }
        );

        let imports_info = imports_info.exclude_imports(vec![(root_package_init, a)])?;

        assert_eq!(
            imports_info.internal_imports,
            hashmap! {
                root_package => hashset! {root_package_init},
                root_package_init => hashset! {b},
                a => hashset! {},
                b => hashset!{},
            }
        );

        assert_eq!(
            imports_info.reverse_internal_imports,
            hashmap! {
                root_package => hashset!{},
                root_package_init => hashset! {root_package},
                a => hashset! {},
                b => hashset! {root_package_init},
            }
        );

        assert_eq!(
            imports_info.internal_imports_metadata,
            hashmap! {
                (root_package_init, b) => ImportMetadata{
                    line_number: 3,
                    is_typechecking: false,
                },
            }
        );

        Ok(())
    }

    #[test]
    fn test_exclude_typechecking_imports() -> Result<()> {
        let test_package = testpackage! {
            "__init__.py" => "
from typing import TYPE_CHECKING

import testpackage.a

if TYPE_CHECKING:
    from testpackage import b
",
            "a.py" => "",
            "b.py" => ""
        };

        let package_info = PackageInfo::build(test_package.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let root_package = imports_info
            .package_info
            .get_item_by_pypath("testpackage")
            .unwrap()
            .token();
        let root_package_init = imports_info
            .package_info
            .get_item_by_pypath("testpackage.__init__")
            .unwrap()
            .token();
        let a = imports_info
            .package_info
            .get_item_by_pypath("testpackage.a")
            .unwrap()
            .token();
        let b = imports_info
            .package_info
            .get_item_by_pypath("testpackage.b")
            .unwrap()
            .token();

        assert_eq!(
            imports_info.internal_imports,
            hashmap! {
                root_package => hashset! {root_package_init},
                root_package_init => hashset! {a, b},
                a => hashset! {},
                b => hashset!{},
            }
        );

        assert_eq!(
            imports_info.reverse_internal_imports,
            hashmap! {
                root_package => hashset!{},
                root_package_init => hashset! {root_package},
                a => hashset! {root_package_init},
                b => hashset! {root_package_init},
            }
        );

        assert_eq!(
            imports_info.internal_imports_metadata,
            hashmap! {
                (root_package_init, a) => ImportMetadata{
                    line_number: 4,
                    is_typechecking: false,
                },
                (root_package_init, b) => ImportMetadata{
                    line_number: 7,
                    is_typechecking: true,
                },
            }
        );

        let imports_info = imports_info.exclude_typechecking_imports()?;

        assert_eq!(
            imports_info.internal_imports,
            hashmap! {
                root_package => hashset! {root_package_init},
                root_package_init => hashset! {a},
                a => hashset! {},
                b => hashset!{},
            }
        );

        assert_eq!(
            imports_info.reverse_internal_imports,
            hashmap! {
                root_package => hashset!{},
                root_package_init => hashset! {root_package},
                a => hashset! {root_package_init},
                b => hashset! {},
            }
        );

        assert_eq!(
            imports_info.internal_imports_metadata,
            hashmap! {
                (root_package_init, a) => ImportMetadata{
                    line_number: 4,
                    is_typechecking: false,
                },
            }
        );

        Ok(())
    }
}
