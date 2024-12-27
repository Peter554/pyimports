use std::collections::HashSet;

use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{ImportsInfo, PackageItemToken};

pub struct InternalImportsQueries<'a> {
    pub(crate) imports_info: &'a ImportsInfo,
}

impl<'a> InternalImportsQueries<'a> {
    pub fn get_items_directly_imported_by(
        &'a self,
        item: PackageItemToken,
    ) -> Result<HashSet<PackageItemToken>> {
        self.get_items(item, |item| {
            Ok(self
                .imports_info
                .internal_imports
                .get(&item)
                .unwrap()
                .clone())
        })
    }

    pub fn get_items_that_directly_import(
        &'a self,
        item: PackageItemToken,
    ) -> Result<HashSet<PackageItemToken>> {
        self.get_items(item, |item| {
            Ok(self
                .imports_info
                .reverse_internal_imports
                .get(&item)
                .unwrap()
                .clone())
        })
    }

    fn get_items<F: Fn(PackageItemToken) -> Result<HashSet<PackageItemToken>> + Send + Sync>(
        &'a self,
        item: PackageItemToken,
        f: F,
    ) -> Result<HashSet<PackageItemToken>> {
        match item {
            PackageItemToken::Package(package) => {
                let package = self.imports_info.package_info.get_package(package)?;

                let package_contents = self
                    .imports_info
                    .package_info
                    .get_package_contents(package.token);

                let mut hs: HashSet<PackageItemToken> = HashSet::new();
                hs.extend(
                    package_contents
                        .all_items
                        .par_iter()
                        .try_fold(
                            HashSet::new,
                            |mut hs, item| -> Result<HashSet<PackageItemToken>> {
                                hs.extend(f(*item)?);
                                Ok(hs)
                            },
                        )
                        .try_reduce(HashSet::new, |mut hs, v| {
                            hs.extend(v);
                            Ok(hs)
                        })?,
                );
                hs = &hs - &package_contents.all_items;

                Ok(hs)
            }
            PackageItemToken::Module(_) => f(item),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use maplit::hashset;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{testpackage, testutils::TestPackage, PackageInfo};

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
        let fruit = imports_info
            .package_info
            .get_item_by_pypath("testpackage.fruit")
            .unwrap()
            .token();
        let colors_package = imports_info
            .package_info
            .get_item_by_pypath("testpackage.colors")
            .unwrap()
            .token();
        let red = imports_info
            .package_info
            .get_item_by_pypath("testpackage.colors.red")
            .unwrap()
            .token();

        // A module
        let imports = imports_info
            .internal_imports()
            .get_items_directly_imported_by(root_package_init)
            .unwrap();
        assert_eq!(imports, hashset! {fruit, red},);

        // A package (removes internal items)
        let imports = imports_info
            .internal_imports()
            .get_items_directly_imported_by(root_package)
            .unwrap();
        assert_eq!(imports, hashset! {},);

        // Another package (removes internal items)
        let imports = imports_info
            .internal_imports()
            .get_items_directly_imported_by(colors_package)
            .unwrap();
        assert_eq!(imports, hashset! {fruit},);

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

        let root_package_init = imports_info
            .package_info
            .get_item_by_pypath("testpackage.__init__")
            .unwrap()
            .token();
        let fruit = imports_info
            .package_info
            .get_item_by_pypath("testpackage.fruit")
            .unwrap()
            .token();
        let colors_package = imports_info
            .package_info
            .get_item_by_pypath("testpackage.colors")
            .unwrap()
            .token();
        let colors_package_init = imports_info
            .package_info
            .get_item_by_pypath("testpackage.colors.__init__")
            .unwrap()
            .token();

        // A module
        let imports = imports_info
            .internal_imports()
            .get_items_that_directly_import(fruit)
            .unwrap();
        assert_eq!(imports, hashset! {root_package_init, colors_package_init},);

        // A package (removes internal items)
        let imports = imports_info
            .internal_imports()
            .get_items_that_directly_import(colors_package)
            .unwrap();
        assert_eq!(imports, hashset! {root_package_init, fruit},);

        Ok(())
    }
}
