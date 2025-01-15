//! The `imports_info` module provides a rich representation of the imports within a python package.
//! See [`ImportsInfo`].
mod queries;

#[allow(dead_code)]
#[doc(hidden)]
#[cfg(feature = "grimp_compare")]
pub(crate) mod grimp_compare;

use crate::errors::Error;
pub use crate::imports_info::queries::external_imports::{
    ExternalImportsPathQuery, ExternalImportsPathQueryBuilder,
    ExternalImportsPathQueryBuilderError, ExternalImportsQueries,
};
pub use crate::imports_info::queries::internal_imports::{
    InternalImportsPathQuery, InternalImportsPathQueryBuilder,
    InternalImportsPathQueryBuilderError, InternalImportsQueries,
};
use crate::package_info::{PackageInfo, PackageItemToken};
use crate::parse;
use crate::parse::resolve_import;
use crate::prelude::*;
use crate::pypath::Pypath;
use anyhow::Result;
use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Metadata associated with an import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportMetadata {
    /// An explicit import.
    ExplicitImport {
        /// The line number of the import statement.
        line_number: usize,
        /// Whether the import statement is for typechecking only (`typing.TYPE_CHECKING`).
        is_typechecking: bool,
    },
    /// An implicit import. E.g. all packages implicitly import their init modules.
    ImplicitImport,
}

/// A rich representation of the imports within a python package.
///
/// ```
/// # use std::collections::HashSet;
/// # use anyhow::Result;
/// # use maplit::{hashmap, hashset};
/// # use pyimports::testpackage;
/// # use pyimports::testutils::TestPackage;
/// use pyimports::package_info::PackageInfo;
/// use pyimports::imports_info::ImportsInfo;
///
/// # fn main() -> Result<()> {
/// let testpackage = testpackage! {
///     "__init__.py" => "from testpackage import a",
///     "a.py" => "from django.db import models"
/// };
///
/// let package_info = PackageInfo::build(testpackage.path())?;
/// let imports_info = ImportsInfo::build(package_info)?;
///
/// let root_pkg = imports_info.package_info()
///     .get_item_by_pypath(&"testpackage".parse()?).unwrap()
///     .token();
/// let root_init = imports_info.package_info()
///     .get_item_by_pypath(&"testpackage.__init__".parse()?).unwrap()
///     .token();
/// let a = imports_info.package_info()
///     .get_item_by_pypath(&"testpackage.a".parse()?).unwrap()
///     .token();
///
/// assert_eq!(
///     imports_info.internal_imports().get_direct_imports(),
///     hashmap! {
///         root_pkg => hashset!{root_init},
///         root_init => hashset!{a},
///         a => hashset!{},
///     }
/// );
///
/// assert_eq!(
///     imports_info.external_imports().get_direct_imports(),
///     hashmap! {
///         root_pkg => hashset!{},
///         root_init => hashset!{},
///         a => hashset!{"django.db.models".parse()?},
///     }
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ImportsInfo {
    // Use `Arc` to avoid cloning `package_info` on `import_info.clone()`.
    package_info: Arc<PackageInfo>,
    //
    internal_imports: HashMap<PackageItemToken, HashSet<PackageItemToken>>,
    reverse_internal_imports: HashMap<PackageItemToken, HashSet<PackageItemToken>>,
    internal_imports_metadata: HashMap<(PackageItemToken, PackageItemToken), ImportMetadata>,
    //
    external_imports: HashMap<PackageItemToken, HashSet<Pypath>>,
    external_imports_metadata: HashMap<(PackageItemToken, Pypath), ImportMetadata>,
}

/// Options for building an [`ImportsInfo`].
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
    /// Creates (default) build options.
    pub fn new() -> Self {
        ImportsInfoBuildOptions {
            include_typechecking_imports: true,
            include_external_imports: true,
        }
    }

    /// Typechecking imports (`typing.TYPE_CHECKING`) should be excluded.
    pub fn with_typechecking_imports_excluded(mut self) -> Self {
        self.include_typechecking_imports = false;
        self
    }

    /// External imports should be excluded.
    pub fn with_external_imports_excluded(mut self) -> Self {
        self.include_external_imports = false;
        self
    }
}

impl ImportsInfo {
    /// Builds an [`ImportsInfo`] with the default options.
    pub fn build(package_info: PackageInfo) -> Result<Self> {
        ImportsInfo::build_with_options(package_info, ImportsInfoBuildOptions::new())
    }

    /// Builds an [`ImportsInfo`] with custom options.
    pub fn build_with_options(
        package_info: PackageInfo,
        options: ImportsInfoBuildOptions,
    ) -> Result<Self> {
        let package_info = Arc::new(package_info);

        let all_raw_imports = get_all_raw_imports(&package_info)?;

        let mut imports_info = ImportsInfo {
            package_info: Arc::clone(&package_info),
            internal_imports: HashMap::new(),
            reverse_internal_imports: HashMap::new(),
            internal_imports_metadata: HashMap::new(),
            external_imports: HashMap::new(),
            external_imports_metadata: HashMap::new(),
        };

        imports_info.initialise_maps()?;

        // By definition, packages import their init modules.
        for package in package_info.get_all_items().filter_packages() {
            if let Some(init_module) = package.init_module() {
                imports_info.add_internal_import(
                    package.token().into(),
                    init_module.into(),
                    ImportMetadata::ImplicitImport,
                )?;
            }
        }

        for (item, raw_imports) in all_raw_imports {
            for raw_import in raw_imports {
                if !options.include_typechecking_imports && raw_import.is_typechecking {
                    continue;
                }

                let metadata = ImportMetadata::ExplicitImport {
                    line_number: raw_import.line_number,
                    is_typechecking: raw_import.is_typechecking,
                };

                if raw_import.pypath.is_internal(&package_info) {
                    let internal_item = {
                        if let Some(item) = package_info
                            .get_item_by_pypath(&raw_import.pypath)
                            .map(|item| item.token())
                        {
                            // An imported module.
                            item
                        } else if let Some(parent_pypath) = &raw_import.pypath.parent() {
                            if let Some(item) = package_info
                                .get_item_by_pypath(parent_pypath)
                                .map(|item| item.token())
                            {
                                // An imported module member.
                                // e.g. from testpackage.foo import FooClass
                                // The pypath is testpackage.foo.FooClass, so we need to strip the final part.
                                item
                            } else {
                                return Err(Error::UnknownInternalImport(raw_import.pypath))?;
                            }
                        } else {
                            return Err(Error::UnknownInternalImport(raw_import.pypath))?;
                        }
                    };

                    imports_info.add_internal_import(item, internal_item, metadata)?;
                } else if options.include_external_imports {
                    imports_info.add_external_import(item, raw_import.pypath, metadata)?;
                }
            }
        }

        Ok(imports_info)
    }

    /// Returns a reference to the contained [`PackageInfo`].
    pub fn package_info(&self) -> &PackageInfo {
        &self.package_info
    }

    /// Returns an [`InternalImportsQueries`] object, that allows querying internal imports.
    pub fn internal_imports(&self) -> InternalImportsQueries {
        InternalImportsQueries { imports_info: self }
    }

    /// Returns an [`ExternalImportsQueries`] object, that allows querying external imports.
    pub fn external_imports(&self) -> ExternalImportsQueries {
        ExternalImportsQueries { imports_info: self }
    }

    /// Removes the passed imports.
    pub fn remove_imports(
        &mut self,
        internal: impl IntoIterator<Item = (PackageItemToken, PackageItemToken)>,
        external: impl IntoIterator<Item = (PackageItemToken, Pypath)>,
    ) -> Result<()> {
        for (from, to) in internal {
            self.remove_internal_import(from, to)?;
        }
        for (from, to) in external {
            self.remove_external_import(from, to)?;
        }
        Ok(())
    }

    /// Removes typechecking imports.
    pub fn remove_typechecking_imports(&mut self) -> Result<()> {
        let internal_imports = self
            .internal_imports_metadata
            .iter()
            .filter_map(|((from, to), metadata)| match metadata {
                ImportMetadata::ExplicitImport {
                    is_typechecking, ..
                } => {
                    if *is_typechecking {
                        Some((*from, *to))
                    } else {
                        None
                    }
                }
                ImportMetadata::ImplicitImport => None,
            })
            .collect::<HashSet<_>>();

        let external_imports = self
            .external_imports_metadata
            .iter()
            .filter_map(|((from, to), metadata)| match metadata {
                ImportMetadata::ExplicitImport {
                    is_typechecking, ..
                } => {
                    if *is_typechecking {
                        Some((*from, to.clone()))
                    } else {
                        None
                    }
                }
                ImportMetadata::ImplicitImport => None,
            })
            .collect::<HashSet<_>>();

        self.remove_imports(internal_imports, external_imports)?;
        Ok(())
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
        metadata: ImportMetadata,
    ) -> Result<()> {
        self.internal_imports.entry(from).or_default().insert(to);
        self.reverse_internal_imports
            .entry(to)
            .or_default()
            .insert(from);
        self.internal_imports_metadata.insert((from, to), metadata);
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
        to: Pypath,
        metadata: ImportMetadata,
    ) -> Result<()> {
        self.external_imports
            .entry(from)
            .or_default()
            .insert(to.clone());
        self.external_imports_metadata.insert((from, to), metadata);
        Ok(())
    }

    fn remove_external_import(&mut self, from: PackageItemToken, to: Pypath) -> Result<()> {
        if self.external_imports.contains_key(&from) {
            self.external_imports.entry(from).or_default().remove(&to);
        };
        self.external_imports_metadata.remove(&(from, to));
        Ok(())
    }
}

#[derive(Debug)]
struct ResolvedRawImport {
    pypath: Pypath,
    line_number: usize,
    is_typechecking: bool,
}

fn get_all_raw_imports(
    package_info: &PackageInfo,
) -> Result<HashMap<PackageItemToken, Vec<ResolvedRawImport>>> {
    let all_raw_imports = package_info
        .get_all_items()
        .filter_modules()
        .par_bridge()
        .try_fold(
            HashMap::new,
            |mut hm: HashMap<PackageItemToken, Vec<ResolvedRawImport>>, module| -> Result<_> {
                // Parse the raw imports.
                let raw_imports = parse::parse_imports(module.path())?;

                // Resolve any relative imports.
                let raw_imports = raw_imports
                    .into_iter()
                    .map(|raw_import| ResolvedRawImport {
                        pypath: resolve_import(
                            raw_import.pypath(),
                            module.path(),
                            package_info.get_root().path(),
                        )
                        .unwrap_or_else(|_| {
                            panic!("Failed to resolve import: {}", raw_import.pypath())
                        }),
                        line_number: raw_import.line_number(),
                        is_typechecking: raw_import.is_typechecking(),
                    })
                    .collect::<Vec<_>>();

                hm.entry(module.token().into())
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
        let testpackage = testpackage! {
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

        let package_info = PackageInfo::build(testpackage.path())?;
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
                (root_package, root_package_init) => ImportMetadata::ImplicitImport,
                (root_package_init, a) => ImportMetadata::ExplicitImport {
                    line_number: 2,
                    is_typechecking: false,
                },
                (root_package_init, b) => ImportMetadata::ExplicitImport{
                    line_number: 3,
                    is_typechecking: false,
                },
                (a, b) => ImportMetadata::ExplicitImport{
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
                (b, "django.db.models".parse()?) => ImportMetadata::ExplicitImport{
                    line_number: 2,
                    is_typechecking: false,
                },
            }
        );

        Ok(())
    }

    #[test]
    fn test_remove_imports() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "
import testpackage.a
from testpackage import b
",
            "a.py" => "",
            "b.py" => ""
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let mut imports_info = ImportsInfo::build(package_info)?;

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
                (root_package, root_package_init) => ImportMetadata::ImplicitImport,
                (root_package_init, a) => ImportMetadata::ExplicitImport{
                    line_number: 2,
                    is_typechecking: false,
                },
                (root_package_init, b) => ImportMetadata::ExplicitImport{
                    line_number: 3,
                    is_typechecking: false,
                },
            }
        );

        imports_info.remove_imports(vec![(root_package_init, a)], vec![])?;

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
                (root_package, root_package_init) => ImportMetadata::ImplicitImport,
                (root_package_init, b) => ImportMetadata::ExplicitImport{
                    line_number: 3,
                    is_typechecking: false,
                },
            }
        );

        Ok(())
    }

    #[test]
    fn test_remove_typechecking_imports() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "
from typing import TYPE_CHECKING

import testpackage.a

if TYPE_CHECKING:
    from testpackage import b
",
            "a.py" => "",
            "b.py" => ""
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let mut imports_info = ImportsInfo::build(package_info)?;

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
                (root_package, root_package_init) => ImportMetadata::ImplicitImport,
                (root_package_init, a) => ImportMetadata::ExplicitImport{
                    line_number: 4,
                    is_typechecking: false,
                },
                (root_package_init, b) => ImportMetadata::ExplicitImport{
                    line_number: 7,
                    is_typechecking: true,
                },
            }
        );

        imports_info.remove_typechecking_imports()?;

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
                (root_package, root_package_init) => ImportMetadata::ImplicitImport,
                (root_package_init, a) => ImportMetadata::ExplicitImport{
                    line_number: 4,
                    is_typechecking: false,
                },
            }
        );

        Ok(())
    }

    #[test]
    fn test_can_build_for_django() -> Result<()> {
        let package_info = PackageInfo::build("vendor/django/django")?;
        let _ = ImportsInfo::build(package_info)?;
        Ok(())
    }
}
