mod parse;
mod queries;

use anyhow::Result;
use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

pub use crate::imports_info::queries::external_imports::ExternalImportsQueries;
pub use crate::imports_info::queries::internal_imports::InternalImportsQueries;
use crate::{
    package_info::{PackageInfo, PackageItemToken},
    AbsolutePyPath, Error,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportMetadata {
    line_number: usize,
    is_typechecking: bool,
}

pub type PackageItemTokenSet = HashSet<PackageItemToken>;

impl From<PackageItemToken> for PackageItemTokenSet {
    fn from(value: PackageItemToken) -> Self {
        PackageItemTokenSet::from([value])
    }
}

#[derive(Debug, Clone)]
pub struct ImportsInfo {
    package_info: PackageInfo,
    //
    internal_imports: HashMap<PackageItemToken, HashSet<PackageItemToken>>,
    reverse_internal_imports: HashMap<PackageItemToken, HashSet<PackageItemToken>>,
    internal_imports_metadata: HashMap<(PackageItemToken, PackageItemToken), ImportMetadata>,
    //
    external_imports: HashMap<PackageItemToken, HashSet<AbsolutePyPath>>,
    external_imports_metadata: HashMap<(PackageItemToken, AbsolutePyPath), ImportMetadata>,
}

#[derive(Debug, Clone)]
pub struct ImportsInfoBuildOptions {
    include_typechecking_imports: bool,
    include_external_imports: bool,
}

impl Default for ImportsInfoBuildOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl ImportsInfoBuildOptions {
    pub fn new() -> Self {
        ImportsInfoBuildOptions {
            include_typechecking_imports: true,
            include_external_imports: true,
        }
    }

    pub fn exclude_typechecking_imports(mut self) -> Self {
        self.include_typechecking_imports = false;
        self
    }

    pub fn exclude_external_imports(mut self) -> Self {
        self.include_external_imports = false;
        self
    }
}

impl ImportsInfo {
    pub fn build(package_info: PackageInfo) -> Result<Self> {
        ImportsInfo::build_with_options(package_info, ImportsInfoBuildOptions::new())
    }

    pub fn build_with_options(
        package_info: PackageInfo,
        options: ImportsInfoBuildOptions,
    ) -> Result<Self> {
        let all_raw_imports = get_all_raw_imports(&package_info)?;

        let mut imports_info = ImportsInfo {
            package_info: package_info.clone(),
            internal_imports: HashMap::new(),
            reverse_internal_imports: HashMap::new(),
            internal_imports_metadata: HashMap::new(),
            external_imports: HashMap::new(),
            external_imports_metadata: HashMap::new(),
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
                if !options.include_typechecking_imports && raw_import.is_typechecking {
                    continue;
                }

                let metadata = ImportMetadata {
                    line_number: raw_import.line_number,
                    is_typechecking: raw_import.is_typechecking,
                };

                if package_info.pypath_is_internal(&raw_import.pypath) {
                    let internal_item = {
                        if let Some(item) = package_info
                            .get_item_by_pypath(&raw_import.pypath)
                            .map(|item| item.token())
                        {
                            // An imported module.
                            item
                        } else if let Some(item) = package_info
                            .get_item_by_pypath(&raw_import.pypath.parent())
                            .map(|item| item.token())
                        {
                            // An imported module member.
                            // e.g. from testpackage.foo import FooClass
                            // The pypath is testpackage.foo.FooClass, so we need to strip the final part.
                            item
                        } else {
                            return Err(Error::UnknownInternalImport(raw_import.pypath))?;
                        }
                    };

                    imports_info.add_internal_import(item, internal_item, Some(metadata))?;
                } else if options.include_external_imports {
                    imports_info.add_external_import(item, raw_import.pypath, Some(metadata))?;
                }
            }
        }

        Ok(imports_info)
    }

    pub fn package_info(&self) -> &PackageInfo {
        &self.package_info
    }

    pub fn internal_imports(&self) -> InternalImportsQueries {
        InternalImportsQueries { imports_info: self }
    }

    pub fn external_imports(&self) -> ExternalImportsQueries {
        ExternalImportsQueries { imports_info: self }
    }

    pub fn exclude_internal_imports(
        &self,
        imports: impl IntoIterator<Item = (PackageItemToken, PackageItemToken)>,
    ) -> Result<Self> {
        let mut imports_info = self.clone();
        for (from, to) in imports {
            imports_info.remove_internal_import(from, to)?;
        }
        Ok(imports_info)
    }

    pub fn exclude_external_imports(
        &self,
        imports: impl IntoIterator<Item = (PackageItemToken, AbsolutePyPath)>,
    ) -> Result<Self> {
        let mut imports_info = self.clone();
        for (from, to) in imports {
            imports_info.remove_external_import(from, to)?;
        }
        Ok(imports_info)
    }

    pub fn exclude_typechecking_imports(&self) -> Result<Self> {
        let mut imports_info = self.clone();

        let internal_imports =
            self.internal_imports_metadata
                .iter()
                .filter_map(|((from, to), metadata)| {
                    if metadata.is_typechecking {
                        Some((*from, *to))
                    } else {
                        None
                    }
                });
        imports_info = imports_info.exclude_internal_imports(internal_imports)?;

        let external_imports =
            self.external_imports_metadata
                .iter()
                .filter_map(|((from, to), metadata)| {
                    if metadata.is_typechecking {
                        Some((*from, to.clone()))
                    } else {
                        None
                    }
                });
        imports_info = imports_info.exclude_external_imports(external_imports)?;

        Ok(imports_info)
    }

    fn initialise_maps(&mut self) -> Result<()> {
        for item in self.package_info.get_all_items() {
            self.internal_imports.entry(item.token()).or_default();
            self.reverse_internal_imports
                .entry(item.token())
                .or_default();
            //
            self.external_imports.entry(item.token()).or_default();
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
        if self.internal_imports.contains_key(&from) {
            self.internal_imports.entry(from).or_default().remove(&to);
        }
        if self.reverse_internal_imports.contains_key(&to) {
            self.reverse_internal_imports
                .entry(to)
                .or_default()
                .remove(&from);
        }
        self.internal_imports_metadata.remove(&(from, to));
        Ok(())
    }

    fn add_external_import(
        &mut self,
        from: PackageItemToken,
        to: AbsolutePyPath,
        metadata: Option<ImportMetadata>,
    ) -> Result<()> {
        self.external_imports
            .entry(from)
            .or_default()
            .insert(to.clone());
        if let Some(metadata) = metadata {
            self.external_imports_metadata.insert((from, to), metadata);
        }
        Ok(())
    }

    fn remove_external_import(&mut self, from: PackageItemToken, to: AbsolutePyPath) -> Result<()> {
        if self.external_imports.contains_key(&from) {
            self.external_imports.entry(from).or_default().remove(&to);
        };
        self.external_imports_metadata.remove(&(from, to));
        Ok(())
    }
}

#[derive(Debug)]
struct ResolvedRawImport {
    pypath: AbsolutePyPath,
    line_number: usize,
    is_typechecking: bool,
}

fn get_all_raw_imports(
    package_info: &PackageInfo,
) -> Result<HashMap<PackageItemToken, Vec<ResolvedRawImport>>> {
    let all_raw_imports = package_info
        .get_all_items()
        .filter_map(PackageInfo::filter_modules)
        .par_bridge()
        .try_fold(
            HashMap::new,
            |mut hm: HashMap<PackageItemToken, Vec<ResolvedRawImport>>, module| -> Result<_> {
                // Parse the raw imports.
                let raw_imports = parse::parse_imports(&module.path)?;

                // Resolve any relative imports.
                let raw_imports = raw_imports
                    .into_iter()
                    .map(|o| ResolvedRawImport {
                        pypath: o
                            .pypath
                            .resolve_relative(&module.path, &package_info.get_root().path),
                        line_number: o.line_number,
                        is_typechecking: o.is_typechecking,
                    })
                    .collect::<Vec<_>>();

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{testpackage, testutils::TestPackage};
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

        let root_package = imports_info._item("testpackage");
        let root_package_init = imports_info._item("testpackage.__init__");
        let a = imports_info._item("testpackage.a");
        let b = imports_info._item("testpackage.b");

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

        assert_eq!(
            imports_info.external_imports,
            hashmap! {
                root_package => hashset! {},
                root_package_init => hashset! {},
                a => hashset! {},
                b => hashset!{"django.db.models".parse()?},
            }
        );

        assert_eq!(
            imports_info.external_imports_metadata,
            hashmap! {
                (b, "django.db.models".parse()?) => ImportMetadata{
                    line_number: 2,
                    is_typechecking: false,
                },
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

        let root_package = imports_info._item("testpackage");
        let root_package_init = imports_info._item("testpackage.__init__");
        let a = imports_info._item("testpackage.a");
        let b = imports_info._item("testpackage.b");

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

        let imports_info = imports_info.exclude_internal_imports(vec![(root_package_init, a)])?;

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

        let root_package = imports_info._item("testpackage");
        let root_package_init = imports_info._item("testpackage.__init__");
        let a = imports_info._item("testpackage.a");
        let b = imports_info._item("testpackage.b");

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
