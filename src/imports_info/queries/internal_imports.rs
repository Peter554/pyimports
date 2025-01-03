use std::collections::{HashMap, HashSet};

use anyhow::Result;
use pathfinding::prelude::{bfs, bfs_reach};

use crate::{Error, ImportMetadata, ImportsInfo, PackageItemToken, PackageItemTokenSet};

/// An object that allows querying internal imports.
pub struct InternalImportsQueries<'a> {
    pub(crate) imports_info: &'a ImportsInfo,
}

#[derive(Debug, Clone)]
/// An object used to build an internal imports path query.
pub struct InternalImportsPathQuery {
    from: PackageItemTokenSet,
    to: PackageItemTokenSet,
    excluding_paths_via: PackageItemTokenSet,
}

impl Default for InternalImportsPathQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl InternalImportsPathQuery {
    /// Creates a new [InternalImportsPathQuery].
    pub fn new() -> Self {
        InternalImportsPathQuery {
            from: PackageItemTokenSet::new(),
            to: PackageItemTokenSet::new(),
            excluding_paths_via: PackageItemTokenSet::new(),
        }
    }

    /// Adds package items from which paths may start.
    pub fn from<T: Into<PackageItemTokenSet>>(mut self, items: T) -> Self {
        let items: PackageItemTokenSet = items.into();
        self.from.extend(items);
        self
    }

    /// Adds package items to which paths may end.
    pub fn to<T: Into<PackageItemTokenSet>>(mut self, items: T) -> Self {
        let items: PackageItemTokenSet = items.into();
        self.to.extend(items);
        self
    }

    /// Exclude paths that would go via the passed package items.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage,TestPackage,PackageInfo,ImportsInfo,InternalImportsPathQuery};
    /// # fn main() -> Result<()> {
    /// let test_package = testpackage! {
    /// "__init__.py" => "",
    ///     "a.py" => "from testpackage import b, e",
    ///     "b.py" => "from testpackage import c",
    ///     "c.py" => "",
    ///     "d.py" => "from testpackage import c",
    ///     "e.py" => "from testpackage import d"
    /// };
    ///
    /// let package_info = PackageInfo::build(test_package.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.a")?.unwrap()
    ///     .token();
    /// let b = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.b")?.unwrap()
    ///     .token();
    /// let c = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.c")?.unwrap()
    ///     .token();
    /// let d = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.d")?.unwrap()
    ///     .token();
    /// let e = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.e")?.unwrap()
    ///     .token();
    ///
    /// // Sanity check: The shortest path goes via `b`.
    /// assert_eq!(
    ///     imports_info.internal_imports().find_path(
    ///         &InternalImportsPathQuery::new()
    ///             .from(a)
    ///             .to(c)
    ///     )?,
    ///     Some(vec![a, b, c])
    /// );
    ///
    /// // If we exclude `b`, we get the longer path via `e`.
    /// assert_eq!(
    ///     imports_info.internal_imports().find_path(
    ///         &InternalImportsPathQuery::new()
    ///             .from(a)
    ///             .to(c)
    ///             .excluding_paths_via(b)
    ///     )?,
    ///     Some(vec![a, e, d, c])
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn excluding_paths_via<T: Into<PackageItemTokenSet>>(mut self, items: T) -> Self {
        let items: PackageItemTokenSet = items.into();
        self.excluding_paths_via.extend(items);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum PathfindingNode {
    Initial,
    PackageItem(PackageItemToken),
}

impl<'a> InternalImportsQueries<'a> {
    /// Returns a map of all the direct imports.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage,TestPackage,PackageInfo,ImportsInfo};
    /// # fn main() -> Result<()> {
    /// let test_package = testpackage! {
    ///     "__init__.py" => "from testpackage import a",
    ///     "a.py" => "from django.db import models"
    /// };
    ///
    /// let package_info = PackageInfo::build(test_package.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let root_pkg = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage")?.unwrap()
    ///     .token();
    /// let root_init = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.__init__")?.unwrap()
    ///     .token();
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.a")?.unwrap()
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
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_direct_imports(&self) -> HashMap<PackageItemToken, HashSet<PackageItemToken>> {
        self.imports_info.internal_imports.clone()
    }

    /// Returns true if a direct import exists.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage,TestPackage,PackageInfo,ImportsInfo};
    /// # fn main() -> Result<()> {
    /// let test_package = testpackage! {
    ///     "__init__.py" => "from testpackage import a",
    ///     "a.py" => "from django.db import models"
    /// };
    ///
    /// let package_info = PackageInfo::build(test_package.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let root_init = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.__init__")?.unwrap()
    ///     .token();
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.a")?.unwrap()
    ///     .token();
    ///
    /// assert!(
    ///     imports_info.internal_imports().direct_import_exists(root_init, a)?,
    /// );
    /// assert!(
    ///     !imports_info.internal_imports().direct_import_exists(a, root_init)?,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn direct_import_exists(
        &self,
        from: PackageItemToken,
        to: PackageItemToken,
    ) -> Result<bool> {
        self.imports_info.package_info.get_item(from)?;
        self.imports_info.package_info.get_item(to)?;

        Ok(self
            .imports_info
            .internal_imports
            .get(&from)
            .unwrap()
            .contains(&to))
    }

    /// Returns the package items directly imported by the passed package item.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage,TestPackage,PackageInfo,ImportsInfo};
    /// # fn main() -> Result<()> {
    /// let test_package = testpackage! {
    ///     "__init__.py" => "from testpackage import a, b",
    ///     "a.py" => "from testpackage import b",
    ///     "b.py" => "from testpackage import c",
    ///     "c.py" => ""
    /// };
    ///
    /// let package_info = PackageInfo::build(test_package.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let root_init = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.__init__")?.unwrap()
    ///     .token();
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.a")?.unwrap()
    ///     .token();
    /// let b = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.b")?.unwrap()
    ///     .token();
    ///
    /// assert_eq!(
    ///     imports_info.internal_imports().get_items_directly_imported_by(root_init)?,
    ///     hashset!{a, b}
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_items_directly_imported_by(
        &'a self,
        item: PackageItemToken,
    ) -> Result<HashSet<PackageItemToken>> {
        self.imports_info.package_info.get_item(item)?;

        Ok(self
            .imports_info
            .internal_imports
            .get(&item)
            .unwrap()
            .clone())
    }

    /// Returns the package items that directly import the passed package item.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage,TestPackage,PackageInfo,ImportsInfo};
    /// # fn main() -> Result<()> {
    /// let test_package = testpackage! {
    ///     "__init__.py" => "from testpackage import a, b",
    ///     "a.py" => "from testpackage import b",
    ///     "b.py" => "from testpackage import c",
    ///     "c.py" => ""
    /// };
    ///
    /// let package_info = PackageInfo::build(test_package.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let root_init = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.__init__")?.unwrap()
    ///     .token();
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.a")?.unwrap()
    ///     .token();
    /// let b = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.b")?.unwrap()
    ///     .token();
    ///
    /// assert_eq!(
    ///     imports_info.internal_imports().get_items_that_directly_import(b)?,
    ///     hashset!{root_init, a}
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_items_that_directly_import(
        &'a self,
        item: PackageItemToken,
    ) -> Result<HashSet<PackageItemToken>> {
        self.imports_info.package_info.get_item(item)?;

        Ok(self
            .imports_info
            .reverse_internal_imports
            .get(&item)
            .unwrap()
            .clone())
    }

    /// Returns the downstream package items.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage,TestPackage,PackageInfo,ImportsInfo};
    /// # fn main() -> Result<()> {
    /// let test_package = testpackage! {
    ///     "__init__.py" => "from testpackage import a, b",
    ///     "a.py" => "from testpackage import b",
    ///     "b.py" => "from testpackage import c",
    ///     "c.py" => ""
    /// };
    ///
    /// let package_info = PackageInfo::build(test_package.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let root_init = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.__init__")?.unwrap()
    ///     .token();
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.a")?.unwrap()
    ///     .token();
    /// let b = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.b")?.unwrap()
    ///     .token();
    /// let c = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.c")?.unwrap()
    ///     .token();
    ///
    /// assert_eq!(
    ///     imports_info.internal_imports().get_downstream_items(root_init)?,
    ///     hashset!{a, b, c}
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_downstream_items<T: Into<PackageItemTokenSet>>(
        &'a self,
        items: T,
    ) -> Result<HashSet<PackageItemToken>> {
        self.bfs_reach(items, &self.imports_info.internal_imports)
    }

    /// Returns the upstream package items.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage,TestPackage,PackageInfo,ImportsInfo};
    /// # fn main() -> Result<()> {
    /// let test_package = testpackage! {
    ///     "__init__.py" => "from testpackage import a, b",
    ///     "a.py" => "from testpackage import b",
    ///     "b.py" => "from testpackage import c",
    ///     "c.py" => ""
    /// };
    ///
    /// let package_info = PackageInfo::build(test_package.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let root_pkg = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage")?.unwrap()
    ///     .token();
    /// let root_init = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.__init__")?.unwrap()
    ///     .token();
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.a")?.unwrap()
    ///     .token();
    /// let b = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.b")?.unwrap()
    ///     .token();
    /// let c = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.c")?.unwrap()
    ///     .token();
    ///
    /// assert_eq!(
    ///     imports_info.internal_imports().get_upstream_items(c)?,
    ///     hashset!{root_pkg, root_init, a, b}
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_upstream_items<T: Into<PackageItemTokenSet>>(
        &'a self,
        items: T,
    ) -> Result<HashSet<PackageItemToken>> {
        self.bfs_reach(items, &self.imports_info.reverse_internal_imports)
    }

    fn bfs_reach<T: Into<PackageItemTokenSet>>(
        &'a self,
        items: T,
        imports_map: &HashMap<PackageItemToken, HashSet<PackageItemToken>>,
    ) -> Result<HashSet<PackageItemToken>> {
        let items: PackageItemTokenSet = items.into();

        for item in items.iter() {
            self.imports_info.package_info.get_item(*item)?;
        }

        let reachable_items = bfs_reach(PathfindingNode::Initial, |item| {
            let items = match item {
                PathfindingNode::Initial => items.clone(),
                PathfindingNode::PackageItem(item) => imports_map.get(item).unwrap().clone(),
            };
            items.into_iter().map(PathfindingNode::PackageItem)
        })
        .filter_map(|item| match item {
            PathfindingNode::Initial => None,
            PathfindingNode::PackageItem(item) => Some(item),
        })
        .collect::<HashSet<_>>();

        let reachable_items = &reachable_items - &items;

        Ok(reachable_items)
    }

    /// Returns the metadata associated with the passed import.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage,TestPackage,PackageInfo,ImportsInfo,ImportMetadata,ExplicitImportMetadata};
    /// # fn main() -> Result<()> {
    /// let test_package = testpackage! {
    ///     "__init__.py" => "from testpackage import a",
    ///     "a.py" => ""
    /// };
    ///
    /// let package_info = PackageInfo::build(test_package.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let root_init = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.__init__")?.unwrap()
    ///     .token();
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.a")?.unwrap()
    ///     .token();
    ///
    /// assert_eq!(
    ///     imports_info.internal_imports().get_import_metadata(root_init, a)?,
    ///     &ImportMetadata::ExplicitImport(ExplicitImportMetadata {
    ///         line_number: 1,
    ///         is_typechecking: false
    ///     })
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_import_metadata(
        &'a self,
        from: PackageItemToken,
        to: PackageItemToken,
    ) -> Result<&'a ImportMetadata> {
        if self.direct_import_exists(from, to)? {
            Ok(self
                .imports_info
                .internal_imports_metadata
                .get(&(from, to))
                .unwrap())
        } else {
            Err(Error::NoSuchImport)?
        }
    }

    /// Returns the shortest import path between the passed package items,
    /// or `None` if no path can be found.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage,TestPackage,PackageInfo,ImportsInfo,InternalImportsPathQuery};
    /// # fn main() -> Result<()> {
    /// let test_package = testpackage! {
    ///     "__init__.py" => "from testpackage import a, b",
    ///     "a.py" => "from testpackage import b",
    ///     "b.py" => "from testpackage import c",
    ///     "c.py" => ""
    /// };
    ///
    /// let package_info = PackageInfo::build(test_package.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let root_init = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.__init__")?.unwrap()
    ///     .token();
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.a")?.unwrap()
    ///     .token();
    /// let b = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.b")?.unwrap()
    ///     .token();
    /// let c = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.c")?.unwrap()
    ///     .token();
    ///
    /// assert_eq!(
    ///     imports_info.internal_imports().find_path(
    ///         &InternalImportsPathQuery::new()
    ///             .from(root_init)
    ///             .to(c)
    ///     )?,
    ///     Some(vec![root_init, b, c])
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_path(
        &'a self,
        query: &InternalImportsPathQuery,
    ) -> Result<Option<Vec<PackageItemToken>>> {
        for item in query.from.iter() {
            self.imports_info.package_info.get_item(*item)?;
        }
        for item in query.to.iter() {
            self.imports_info.package_info.get_item(*item)?;
        }
        for item in query.excluding_paths_via.iter() {
            self.imports_info.package_info.get_item(*item)?;
        }

        let path = bfs(
            &PathfindingNode::Initial,
            |item| {
                let items = match item {
                    PathfindingNode::Initial => query.from.clone(),
                    PathfindingNode::PackageItem(item) => self
                        .imports_info
                        .internal_imports
                        .get(item)
                        .unwrap()
                        .clone(),
                };
                let items = &items - &query.excluding_paths_via;
                items.into_iter().map(PathfindingNode::PackageItem)
            },
            |item| match item {
                PathfindingNode::Initial => false,
                PathfindingNode::PackageItem(item) => query.to.contains(item),
            },
        );

        let path = path.map(|path| {
            path.into_iter()
                .skip(1)
                .map(|item| match item {
                    PathfindingNode::PackageItem(item) => item,
                    PathfindingNode::Initial => panic!(),
                })
                .collect()
        });

        Ok(path)
    }

    /// Returns true if an import path exists between the passed package items.
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use anyhow::Result;
    /// # use maplit::{hashmap, hashset};
    /// # use pyimports::{testpackage,TestPackage,PackageInfo,ImportsInfo,InternalImportsPathQuery};
    /// # fn main() -> Result<()> {
    /// let test_package = testpackage! {
    ///     "__init__.py" => "from testpackage import a, b",
    ///     "a.py" => "from testpackage import b",
    ///     "b.py" => "from testpackage import c",
    ///     "c.py" => ""
    /// };
    ///
    /// let package_info = PackageInfo::build(test_package.path())?;
    /// let imports_info = ImportsInfo::build(package_info)?;
    ///
    /// let root_init = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.__init__")?.unwrap()
    ///     .token();
    /// let a = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.a")?.unwrap()
    ///     .token();
    /// let b = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.b")?.unwrap()
    ///     .token();
    /// let c = imports_info.package_info()
    ///     .get_item_by_pypath("testpackage.c")?.unwrap()
    ///     .token();
    ///
    /// assert!(
    ///     imports_info.internal_imports().path_exists(
    ///         &InternalImportsPathQuery::new().from(root_init).to(c)
    ///     )?,
    /// );
    /// assert!(
    ///     !imports_info.internal_imports().path_exists(
    ///         &InternalImportsPathQuery::new().from(c).to(root_init)
    ///     )?,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn path_exists(&'a self, query: &InternalImportsPathQuery) -> Result<bool> {
        Ok(self.find_path(query)?.is_some())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use maplit::{hashmap, hashset};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{testpackage, testutils::TestPackage, Error, ExplicitImportMetadata, PackageInfo};

    #[test]
    fn test_get_direct_imports() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "
import testpackage.fruit
from testpackage import colors
",
            "fruit.py" => "",
            "colors.py" => ""
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let root_package = imports_info._item("testpackage");
        let root_package_init = imports_info._item("testpackage.__init__");
        let fruit = imports_info._item("testpackage.fruit");
        let colors = imports_info._item("testpackage.colors");

        assert_eq!(
            imports_info.internal_imports().get_direct_imports(),
            hashmap! {
                root_package => hashset! {root_package_init},
                root_package_init => hashset! {fruit, colors},
                fruit => hashset! {},
                colors => hashset! {}
            }
        );

        Ok(())
    }

    #[test]
    fn test_get_items_directly_imported_by() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "
import testpackage.fruit
from testpackage.colors import red
",

            "fruit.py" => "",

            "colors/__init__.py" => "
from .. import fruit
from . import red",

            "colors/red.py" => ""
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let root_package_init = imports_info._item("testpackage.__init__");
        let fruit = imports_info._item("testpackage.fruit");
        let red = imports_info._item("testpackage.colors.red");

        let imports = imports_info
            .internal_imports()
            .get_items_directly_imported_by(root_package_init)
            .unwrap();
        assert_eq!(imports, hashset! {fruit, red},);

        Ok(())
    }

    #[test]
    fn test_get_items_that_directly_import() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "
import testpackage.fruit
from testpackage import colors
",

            "fruit.py" => "
from testpackage.colors import red
",

            "colors/__init__.py" => "
from .. import fruit
",

            "colors/red.py" => "
from testpackage import colors
"
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let root_package_init = imports_info._item("testpackage.__init__");
        let fruit = imports_info._item("testpackage.fruit");
        let colors_package_init = imports_info._item("testpackage.colors.__init__");

        let imports = imports_info
            .internal_imports()
            .get_items_that_directly_import(fruit)
            .unwrap();
        assert_eq!(imports, hashset! {root_package_init, colors_package_init},);

        Ok(())
    }

    #[test]
    fn test_get_downstream_items() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",

            "a.py" => "from testpackage import b",
            "b.py" => "from testpackage import c",
            "c.py" => "",

            "d.py" => "from testpackage import e",
            "e.py" => "from testpackage import f",
            "f.py" => ""
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let a = imports_info._item("testpackage.a");
        let b = imports_info._item("testpackage.b");
        let c = imports_info._item("testpackage.c");
        let d = imports_info._item("testpackage.d");
        let e = imports_info._item("testpackage.e");
        let f = imports_info._item("testpackage.f");

        let imports = imports_info
            .internal_imports()
            .get_downstream_items(a)
            .unwrap();
        assert_eq!(imports, hashset! {b, c},);

        let imports = imports_info
            .internal_imports()
            .get_downstream_items(hashset! {a, d})
            .unwrap();
        assert_eq!(imports, hashset! {b, c, e, f},);

        Ok(())
    }

    #[test]
    fn test_get_upstream_items() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "
from testpackage import fruit
",

            "fruit.py" => "
from testpackage import colors
from testpackage import books",

            "colors.py" => "",
            "books.py" => ""
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let root_package = imports_info._item("testpackage");
        let root_package_init = imports_info._item("testpackage.__init__");
        let fruit = imports_info._item("testpackage.fruit");
        let colors = imports_info._item("testpackage.colors");
        let books = imports_info._item("testpackage.books");

        let imports = imports_info
            .internal_imports()
            .get_upstream_items(colors)
            .unwrap();
        assert_eq!(imports, hashset! {root_package,root_package_init, fruit},);

        let imports = imports_info
            .internal_imports()
            .get_upstream_items(books)
            .unwrap();
        assert_eq!(imports, hashset! {root_package,root_package_init, fruit},);

        Ok(())
    }

    #[test]
    fn test_get_import_metadata() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "from testpackage import fruit",
            "fruit.py" => ""
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let root_package = imports_info._item("testpackage");
        let root_package_init = imports_info._item("testpackage.__init__");
        let fruit = imports_info._item("testpackage.fruit");

        let internal_imports = imports_info.internal_imports();

        let metadata = internal_imports.get_import_metadata(root_package, root_package_init)?;
        assert_eq!(metadata, &ImportMetadata::ImplicitImport);

        let metadata = internal_imports.get_import_metadata(root_package_init, fruit)?;
        assert_eq!(
            metadata,
            &ImportMetadata::ExplicitImport(ExplicitImportMetadata {
                line_number: 1,
                is_typechecking: false
            })
        );

        let metadata = internal_imports.get_import_metadata(root_package, fruit);
        assert_eq!(
            metadata.err().unwrap().downcast_ref::<Error>().unwrap(),
            &Error::NoSuchImport
        );

        Ok(())
    }

    #[test]
    fn test_find_path() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "a.py" => "from testpackage import b; from testpackage import c",
            "b.py" => "from testpackage import c",
            "c.py" => "from testpackage import d; from testpackage import e",
            "d.py" => "from testpackage import e",
            "e.py" => ""
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let a = imports_info._item("testpackage.a");
        let c = imports_info._item("testpackage.c");
        let e = imports_info._item("testpackage.e");

        assert_eq!(
            imports_info
                .internal_imports()
                .find_path(&InternalImportsPathQuery::new().from(a).to(e))?,
            Some(vec![a, c, e])
        );

        Ok(())
    }

    #[test]
    fn test_get_path_excluding_paths_via() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "a.py" => "from testpackage import b, e",
            "b.py" => "from testpackage import c",
            "c.py" => "",
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

        assert_eq!(
            imports_info
                .internal_imports()
                .find_path(&InternalImportsPathQuery::new().from(a).to(c))?,
            Some(vec![a, b, c])
        );

        assert_eq!(
            imports_info.internal_imports().find_path(
                &InternalImportsPathQuery::new()
                    .from(a)
                    .to(c)
                    .excluding_paths_via(b)
            )?,
            Some(vec![a, e, d, c])
        );

        Ok(())
    }

    #[test]
    fn test_path_exists() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "a.py" => "from testpackage import b; from testpackage import c",
            "b.py" => "from testpackage import c",
            "c.py" => "from testpackage import d; from testpackage import e",
            "d.py" => "from testpackage import e",
            "e.py" => ""
        };

        let package_info = PackageInfo::build(testpackage.path())?;
        let imports_info = ImportsInfo::build(package_info)?;

        let a = imports_info._item("testpackage.a");
        let e = imports_info._item("testpackage.e");

        assert!(imports_info
            .internal_imports()
            .path_exists(&InternalImportsPathQuery::new().from(a).to(e))?);
        assert!(!imports_info
            .internal_imports()
            .path_exists(&InternalImportsPathQuery::new().from(e).to(a))?);

        Ok(())
    }
}
