use crate::errors::Error;
use crate::imports_info::{ImportMetadata, ImportsInfo};
use crate::package_info::PackageItemToken;
use crate::pypath::Pypath;
use anyhow::Result;
use derive_builder::Builder;
use derive_more::{IsVariant, Unwrap};
use derive_new::new;
use getset::Getters;
use lazy_static::lazy_static;
use pathfinding::prelude::bfs;
use std::collections::{HashMap, HashSet};

lazy_static! {
    static ref EMPTY_PACKAGE_ITEM_TOKENS: HashSet<PackageItemToken> = HashSet::default();
    static ref EMPTY_PYPATHS: HashSet<Pypath> = HashSet::default();
}

/// An object that allows querying external imports.
pub struct ExternalImportsQueries<'a> {
    pub(crate) imports_info: &'a ImportsInfo,
}

/// An object representing an external imports path query.
#[derive(Debug, Clone, new, Getters, Builder)]
#[builder(setter(into))]
pub struct ExternalImportsPathQuery {
    /// Package items from which paths may start.
    #[new(into)]
    #[getset(get = "pub")]
    from: HashSet<PackageItemToken>,

    /// External items where paths may end.
    #[new(into)]
    #[getset(get = "pub")]
    to: HashSet<Pypath>,

    /// Paths that would go via these package items should be excluded.
    #[new(into)]
    #[getset(get = "pub")]
    #[builder(default)]
    excluding_paths_via: HashSet<PackageItemToken>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, IsVariant, Unwrap)]
enum PathfindingNode<'a> {
    Initial,
    PackageItem(&'a PackageItemToken),
    ExternalItem(&'a Pypath),
}

impl<'a> ExternalImportsQueries<'a> {
    /// Returns a map of all the direct imports.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage, testutils::TestPackage};
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
    pub fn get_direct_imports(&self) -> HashMap<PackageItemToken, HashSet<Pypath>> {
        self.imports_info
            .external_imports
            .clone()
            .into_iter()
            .collect()
    }

    /// Returns true if a direct import exists.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage, testutils::TestPackage};
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
    /// let root_init = imports_info.package_info()
    ///     .get_item_by_pypath(&"testpackage.__init__".parse()?).unwrap()
    ///     .token();
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath(&"testpackage.a".parse()?).unwrap()
    ///     .token();
    ///
    /// assert!(
    ///     imports_info.external_imports().direct_import_exists(a, &"django.db.models".parse()?)?,
    /// );
    /// assert!(
    ///     !imports_info.external_imports().direct_import_exists(root_init, &"django.db.models".parse()?)?,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn direct_import_exists(&self, from: PackageItemToken, to: &Pypath) -> Result<bool> {
        self.imports_info.package_info.get_item(from)?;

        Ok(self
            .imports_info
            .external_imports
            .get(from)
            .unwrap()
            .contains(to))
    }

    /// Returns the items directly imported by the passed package item.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage, testutils::TestPackage};
    /// use pyimports::package_info::PackageInfo;
    /// use pyimports::imports_info::ImportsInfo;
    ///
    /// # fn main() -> Result<()> {
    /// let testpackage = testpackage! {
    ///     "__init__.py" => "from testpackage import a",
    ///     "a.py" => "from django.db import models; import pydantic.BaseModel as BM"
    /// };
    ///
    /// let package_info = PackageInfo::build(testpackage.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath(&"testpackage.a".parse()?).unwrap()
    ///     .token();
    ///
    /// assert_eq!(
    ///     imports_info.external_imports().get_items_directly_imported_by(a)?,
    ///     hashset!{
    ///         "django.db.models".parse()?,
    ///         "pydantic.BaseModel".parse()?,
    ///     }
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_items_directly_imported_by(
        &'a self,
        item: PackageItemToken,
    ) -> Result<HashSet<Pypath>> {
        self.imports_info.package_info.get_item(item)?;

        Ok(self
            .imports_info
            .external_imports
            .get(item)
            .unwrap()
            .clone())
    }

    /// Returns the downstream external imports.
    /// This is determined by finding the downstream internal imports and then returning the union
    /// of the external imports from all of these internal items.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage, testutils::TestPackage};
    /// use pyimports::package_info::PackageInfo;
    /// use pyimports::imports_info::ImportsInfo;
    ///
    /// # fn main() -> Result<()> {
    /// let testpackage = testpackage! {
    ///     "__init__.py" => "",
    ///     "a.py" => "from django.db import models; from testpackage import b",
    ///     "b.py" => "import pydantic; from testpackage import c",
    ///     "c.py" => "import numpy as np"
    /// };
    ///
    /// let package_info = PackageInfo::build(testpackage.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath(&"testpackage.a".parse()?).unwrap()
    ///     .token();
    ///
    /// assert_eq!(
    ///     imports_info.external_imports().get_downstream_items(a)?,
    ///     hashset!{
    ///         "django.db.models".parse()?,
    ///         "pydantic".parse()?,
    ///         "numpy".parse()?,
    ///     }
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_downstream_items<T: Into<HashSet<PackageItemToken>>>(
        &'a self,
        items: T,
    ) -> Result<HashSet<Pypath>> {
        let mut items = items.into();

        items.extend(
            self.imports_info
                .internal_imports()
                .get_downstream_items(items.clone())?,
        );

        let external_imports = items
            .into_iter()
            .flat_map(|item| {
                self.imports_info
                    .external_imports
                    .get(item)
                    .unwrap()
                    .clone()
            })
            .collect();

        Ok(external_imports)
    }

    /// Returns the metadata associated with the passed import.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage, testutils::TestPackage};
    /// use pyimports::package_info::PackageInfo;
    /// use pyimports::imports_info::{ImportsInfo,ImportMetadata};
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
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath(&"testpackage.a".parse()?).unwrap()
    ///     .token();
    ///
    /// assert_eq!(
    ///     imports_info.external_imports().get_import_metadata(a, &"django.db.models".parse()?)?,
    ///     &ImportMetadata::ExplicitImport {
    ///         line_number: 1,
    ///         is_typechecking: false
    ///     }
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_import_metadata(
        &'a self,
        from: PackageItemToken,
        to: &Pypath,
    ) -> Result<&'a ImportMetadata> {
        if self.direct_import_exists(from, to)? {
            Ok(self
                .imports_info
                .external_imports_metadata
                .get(&(from, to.clone()))
                .unwrap())
        } else {
            Err(Error::NoSuchImport)?
        }
    }

    /// Returns the shortest import path or `None` if no path can be found.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage, testutils::TestPackage};
    /// use pyimports::package_info::PackageInfo;
    /// use pyimports::imports_info::{ImportsInfo,ImportMetadata,ExternalImportsPathQueryBuilder};
    ///
    /// # fn main() -> Result<()> {
    /// let testpackage = testpackage! {
    ///     "__init__.py" => "",
    ///     "a.py" => "from testpackage import b",
    ///     "b.py" => "from testpackage import c",
    ///     "c.py" => "from django.db import models"
    /// };
    ///
    /// let package_info = PackageInfo::build(testpackage.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath(&"testpackage.a".parse()?).unwrap()
    ///     .token();
    /// let b = imports_info.package_info()
    ///     .get_item_by_pypath(&"testpackage.b".parse()?).unwrap()
    ///     .token();
    /// let c = imports_info.package_info()
    ///     .get_item_by_pypath(&"testpackage.c".parse()?).unwrap()
    ///     .token();
    ///
    /// assert_eq!(
    ///     imports_info.external_imports().find_path(
    ///         &ExternalImportsPathQueryBuilder::default()
    ///             .from(a)
    ///             .to(&"django.db.models".parse()?)
    ///             .build()?
    ///     )?,
    ///     Some((vec![a, b, c], "django.db.models".parse()?))
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_path(
        &'a self,
        query: &ExternalImportsPathQuery,
    ) -> Result<Option<(Vec<PackageItemToken>, Pypath)>> {
        for item in query.from.iter() {
            self.imports_info.package_info.get_item(*item)?;
        }
        for item in query.excluding_paths_via.iter() {
            self.imports_info.package_info.get_item(*item)?;
        }

        let path = bfs(
            &PathfindingNode::Initial,
            // Successors
            |item| {
                let internal_items = match item {
                    PathfindingNode::Initial => &query.from,
                    PathfindingNode::PackageItem(item) => {
                        self.imports_info.internal_imports.get(**item).unwrap()
                    }
                    PathfindingNode::ExternalItem(_) => &EMPTY_PACKAGE_ITEM_TOKENS,
                };

                let external_items = match item {
                    PathfindingNode::Initial => &EMPTY_PYPATHS,
                    PathfindingNode::PackageItem(item) => {
                        self.imports_info.external_imports.get(**item).unwrap()
                    }
                    PathfindingNode::ExternalItem(_) => &EMPTY_PYPATHS,
                };

                let internal_items = internal_items
                    .difference(&query.excluding_paths_via)
                    .map(PathfindingNode::PackageItem);

                let external_items = external_items.iter().map(PathfindingNode::ExternalItem);

                internal_items.chain(external_items)
            },
            // Success
            |item| match item {
                PathfindingNode::Initial => false,
                PathfindingNode::PackageItem(_) => false,
                PathfindingNode::ExternalItem(pypath) => query.to.contains(pypath),
            },
        );

        if path.is_none() {
            return Ok(None);
        }

        let mut path = path.unwrap();
        let external_item = path.pop().unwrap().unwrap_external_item().clone();

        let path = path
            .into_iter()
            .skip(1)
            .map(|item| match item {
                PathfindingNode::PackageItem(item) => item,
                _ => panic!(),
            })
            .cloned()
            .collect::<Vec<_>>();

        Ok(Some((path, external_item)))
    }

    /// Returns true if an import path exists.
    pub fn path_exists(&'a self, query: &ExternalImportsPathQuery) -> Result<bool> {
        Ok(self.find_path(query)?.is_some())
    }

    pub(crate) fn get_equal_to_or_descendant_imports(&self, pypath: &Pypath) -> HashSet<Pypath> {
        self.imports_info
            .external_imports
            .iter()
            .flat_map(|(_, external_imports)| {
                external_imports.iter().filter_map(|imported_pypath| {
                    if imported_pypath.is_equal_to_or_descendant_of(pypath) {
                        Some(imported_pypath.clone())
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use maplit::{hashmap, hashset};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::package_info::PackageInfo;
    use crate::{testpackage, testutils::TestPackage};

    #[test]
    fn test_get_direct_imports() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "import pydantic",
            "a.py" => "from django import db"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let root_package = imports_info._item("testpackage");
        let root_package_init = imports_info._item("testpackage.__init__");
        let a = imports_info._item("testpackage.a");

        assert_eq!(
            imports_info.external_imports().get_direct_imports(),
            hashmap! {
                root_package => hashset!{},
                root_package_init => hashset! {"pydantic".parse()?},
                a => hashset! {"django.db".parse()?},
            }
        );

        Ok(())
    }

    #[test]
    fn test_get_items_directly_imported_by() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "import pydantic",
            "a.py" => "from django import db"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let root_package_init = imports_info._item("testpackage.__init__");

        assert_eq!(
            imports_info
                .external_imports()
                .get_items_directly_imported_by(root_package_init)?,
            hashset! {"pydantic".parse()?}
        );

        Ok(())
    }

    #[test]
    fn test_get_downstream_items() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "a.py" => "from django.db import models; from testpackage import b",
            "b.py" => "import pydantic; from testpackage import c",
            "c.py" => "import numpy as np"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let a = imports_info
            .package_info()
            .get_item_by_pypath(&"testpackage.a".parse()?)
            .unwrap()
            .token();

        assert_eq!(
            imports_info.external_imports().get_downstream_items(a)?,
            hashset! {
                "django.db.models".parse()?,
                "pydantic".parse()?,
                "numpy".parse()?,
            }
        );

        Ok(())
    }

    #[test]
    fn test_get_import_metadata() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "import pydantic"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let root_package_init = imports_info._item("testpackage.__init__");

        let external_imports = imports_info.external_imports();
        let metadata =
            external_imports.get_import_metadata(root_package_init, &"pydantic".parse()?)?;

        assert_eq!(
            metadata,
            &ImportMetadata::ExplicitImport {
                line_number: 1,
                is_typechecking: false
            }
        );

        Ok(())
    }

    #[test]
    fn test_get_equal_to_or_descendant_imports() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "a.py" => "from django.db import models",
            "b.py" => "from django.http import HttpResponse",
            "c.py" => "from django.shortcuts import render"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        assert_eq!(
            imports_info
                .external_imports()
                .get_equal_to_or_descendant_imports(&"django.db.models".parse()?),
            hashset! {
                "django.db.models".parse()?,
            }
        );

        assert_eq!(
            imports_info
                .external_imports()
                .get_equal_to_or_descendant_imports(&"django.db".parse()?),
            hashset! {
                "django.db.models".parse()?,
            }
        );

        assert_eq!(
            imports_info
                .external_imports()
                .get_equal_to_or_descendant_imports(&"django".parse()?),
            hashset! {
                "django.db.models".parse()?,
                "django.http.HttpResponse".parse()?,
                "django.shortcuts.render".parse()?,
            }
        );

        Ok(())
    }

    #[test]
    fn test_find_path() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "a.py" => "from testpackage import b",
            "b.py" => "from testpackage import c",
            "c.py" => "from django.db import models"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let a = imports_info._item("testpackage.a");
        let b = imports_info._item("testpackage.b");
        let c = imports_info._item("testpackage.c");

        assert_eq!(
            imports_info.external_imports().find_path(
                &ExternalImportsPathQueryBuilder::default()
                    .from(a)
                    .to(&"django.db.models".parse()?)
                    .build()?
            )?,
            Some((vec![a, b, c], "django.db.models".parse()?))
        );

        Ok(())
    }

    #[test]
    fn test_find_path_excluding_via() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "a.py" => "from testpackage import b, e",
            "b.py" => "from testpackage import c",
            "c.py" => "from django.db import models",
            "d.py" => "from testpackage import c",
            "e.py" => "from testpackage import d"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let a = imports_info._item("testpackage.a");
        let b = imports_info._item("testpackage.b");
        let c = imports_info._item("testpackage.c");
        let d = imports_info._item("testpackage.d");
        let e = imports_info._item("testpackage.e");

        // Sanity check
        assert_eq!(
            imports_info.external_imports().find_path(
                &ExternalImportsPathQueryBuilder::default()
                    .from(a)
                    .to(&"django.db.models".parse()?)
                    .build()?
            )?,
            Some((vec![a, b, c], "django.db.models".parse()?))
        );

        // Excluding b we need to go via the longer path
        assert_eq!(
            imports_info.external_imports().find_path(
                &ExternalImportsPathQueryBuilder::default()
                    .from(a)
                    .to(&"django.db.models".parse()?)
                    .excluding_paths_via(b)
                    .build()?
            )?,
            Some((vec![a, e, d, c], "django.db.models".parse()?))
        );

        Ok(())
    }
}
