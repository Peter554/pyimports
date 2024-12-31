use std::collections::{HashMap, HashSet};

use anyhow::Result;
use pathfinding::prelude::{bfs, bfs_reach};

use crate::{Error, ImportMetadata, ImportsInfo, PackageItemToken, PackageItemTokenSet};

pub struct InternalImportsQueries<'a> {
    pub(crate) imports_info: &'a ImportsInfo,
}

impl<'a> InternalImportsQueries<'a> {
    pub fn get_direct_imports(&self) -> HashMap<PackageItemToken, HashSet<PackageItemToken>> {
        self.imports_info.internal_imports.clone()
    }

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

    pub fn get_downstream_items(
        &'a self,
        item: PackageItemToken,
    ) -> Result<HashSet<PackageItemToken>> {
        self.imports_info.package_info.get_item(item)?;

        let mut items = bfs_reach(item, |item| {
            self.imports_info
                .internal_imports
                .get(item)
                .unwrap()
                .clone()
        })
        .collect::<HashSet<_>>();

        items.remove(&item);

        Ok(items)
    }

    pub fn get_upstream_items(
        &'a self,
        item: PackageItemToken,
    ) -> Result<HashSet<PackageItemToken>> {
        self.imports_info.package_info.get_item(item)?;

        let mut items = bfs_reach(item, |item| {
            self.imports_info
                .reverse_internal_imports
                .get(item)
                .unwrap()
                .clone()
        })
        .collect::<HashSet<_>>();

        items.remove(&item);

        Ok(items)
    }

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

    pub fn get_shortest_path<To>(
        &'a self,
        from: PackageItemToken,
        to: To,
    ) -> Result<Option<Vec<PackageItemToken>>>
    where
        To: Into<PackageItemTokenSet>,
    {
        let to: PackageItemTokenSet = to.into();

        self.imports_info.package_info.get_item(from)?;
        for to_target in to.iter() {
            self.imports_info.package_info.get_item(*to_target)?;
        }

        let path = bfs(
            &from,
            |item| {
                self.imports_info
                    .internal_imports
                    .get(item)
                    .unwrap()
                    .clone()
            },
            |item| to.contains(item),
        );

        Ok(path)
    }

    pub fn path_exists<To>(&'a self, from: PackageItemToken, to: To) -> Result<bool>
    where
        To: Into<PackageItemTokenSet>,
    {
        Ok(self.get_shortest_path(from, to)?.is_some())
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

        let root_package_init = imports_info._item("testpackage.__init__");
        let fruit = imports_info._item("testpackage.fruit");
        let colors = imports_info._item("testpackage.colors");
        let books = imports_info._item("testpackage.books");

        let imports = imports_info
            .internal_imports()
            .get_downstream_items(root_package_init)
            .unwrap();
        assert_eq!(imports, hashset! {fruit, colors, books},);

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
    fn test_get_shortest_path() -> Result<()> {
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
            imports_info.internal_imports().get_shortest_path(a, e)?,
            Some(vec![a, c, e])
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

        assert!(imports_info.internal_imports().path_exists(a, e)?);
        assert!(!imports_info.internal_imports().path_exists(e, a)?);

        Ok(())
    }
}
