use crate::errors::Error;
use crate::package_info::{
    Module, ModuleToken, Package, PackageInfo, PackageItem, PackageItemToken, PackageToken,
};
use crate::prelude::*;
use anyhow::Result;
use std::borrow::Borrow;
use std::path::Path;

/// An iterator over package items.
pub trait PackageItemIterator<'a>: Iterator<Item = PackageItem<'a>> + Sized {
    /// Filter to packages only.
    ///
    /// ```
    /// # use anyhow::Result;
    /// # use pyimports::{testpackage};
    /// # use pyimports::testutils::TestPackage;
    /// use pyimports::prelude::*;
    /// use pyimports::package_info::{PackageInfo,Package};
    ///
    /// # fn main() -> Result<()> {
    /// # let testpackage = testpackage! {
    /// #     "__init__.py" => ""
    /// # };
    /// # let package_info = PackageInfo::build(testpackage.path()).unwrap();
    /// let packages = package_info
    ///     .get_all_items()
    ///     .filter_packages()
    ///     .collect::<Vec<&Package>>();
    /// # Ok(())
    /// # }
    /// ```
    fn filter_packages(self) -> impl Iterator<Item = &'a Package> {
        self.filter_map(|item| match item {
            PackageItem::Package(package) => Some(package),
            _ => None,
        })
    }

    /// Filter to modules only.
    ///
    /// ```
    /// # use anyhow::Result;
    /// # use pyimports::{testpackage};
    /// # use pyimports::testutils::TestPackage;
    /// use pyimports::prelude::*;
    /// use pyimports::package_info::{PackageInfo,Module};
    ///
    /// # fn main() -> Result<()> {
    /// # let testpackage = testpackage! {
    /// #     "__init__.py" => ""
    /// # };
    /// # let package_info = PackageInfo::build(testpackage.path()).unwrap();
    /// let modules = package_info
    ///     .get_all_items()
    ///     .filter_modules()
    ///     .collect::<Vec<&Module>>();
    /// # Ok(())
    /// # }
    /// ```
    fn filter_modules(self) -> impl Iterator<Item = &'a Module> + Sized {
        self.filter_map(|item| match item {
            PackageItem::Module(module) => Some(module),
            _ => None,
        })
    }
}

impl<'a, T: Iterator<Item = PackageItem<'a>>> PackageItemIterator<'a> for T {}

impl PackageInfo {
    /// Get a package item via the associated filesystem path.
    pub fn get_item_by_path(&self, path: &Path) -> Option<PackageItem> {
        if let Some(package) = self.packages_by_path.get(path) {
            Some(self.get_package(*package).unwrap().into())
        } else {
            self.modules_by_path
                .get(path)
                .map(|module| self.get_module(*module).unwrap().into())
        }
    }

    /// Get a package item via the associated pypath.
    ///
    /// ```
    /// # use anyhow::Result;
    /// # use pyimports::{testpackage};
    /// # use pyimports::testutils::TestPackage;
    /// use pyimports::package_info::PackageInfo;
    ///
    /// # fn main() -> Result<()> {
    /// let testpackage = testpackage! {
    ///     "__init__.py" => "",
    ///     "foo.py" => ""
    /// };
    ///
    /// let package_info = PackageInfo::build(&testpackage.path())?;
    ///
    /// let foo = package_info.get_item_by_pypath("testpackage.foo")?;
    /// assert!(foo.is_some());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_item_by_pypath<T: IntoPypath>(&self, pypath: T) -> Result<Option<PackageItem>> {
        let pypath = pypath.into_pypath()?;
        if let Some(package) = self.packages_by_pypath.get(pypath.borrow()) {
            Ok(Some(self.get_package(*package).unwrap().into()))
        } else {
            Ok(self
                .modules_by_pypath
                .get(pypath.borrow())
                .map(|module| self.get_module(*module).unwrap().into()))
        }
    }

    /// Get a package item via the associated token.
    pub fn get_item(&self, token: PackageItemToken) -> Result<PackageItem> {
        match token {
            PackageItemToken::Package(token) => Ok(self.get_package(token)?.into()),
            PackageItemToken::Module(token) => Ok(self.get_module(token)?.into()),
        }
    }

    /// Get a package via the associated token.
    pub fn get_package(&self, token: PackageToken) -> Result<&Package> {
        match self.packages.get(token) {
            Some(package) => Ok(package),
            None => Err(Error::UnknownPackage(token))?,
        }
    }

    /// Get a module via the associated token.
    pub fn get_module(&self, token: ModuleToken) -> Result<&Module> {
        match self.modules.get(token) {
            Some(module) => Ok(module),
            None => Err(Error::UnknownModule(token))?,
        }
    }

    /// Get the root package.
    pub fn get_root(&self) -> &Package {
        self.get_package(self.root).unwrap()
    }

    /// Get the parent package of the passed package item.
    pub fn get_parent_package(&self, token: PackageItemToken) -> Result<Option<&Package>> {
        let item = self.get_item(token)?;
        let parent = match item {
            PackageItem::Package(package) => match package.parent {
                Some(parent) => Some(self.get_item(parent.into())?),
                None => None,
            },
            PackageItem::Module(module) => Some(self.get_item(module.parent.into())?),
        };
        match parent {
            Some(parent) => match parent {
                PackageItem::Package(parent) => Ok(Some(parent)),
                PackageItem::Module(_) => panic!("Parent is a module?!"),
            },
            None => Ok(None),
        }
    }

    /// Get an iterator over the child items of the passed package.
    ///
    /// ```
    /// # use anyhow::Result;
    /// # use pyimports::{testpackage};
    /// # use pyimports::testutils::TestPackage;
    /// use pyimports::prelude::*;
    /// use pyimports::package_info::{PackageItem,PackageInfo};
    ///
    /// # fn main() -> Result<()> {
    /// # let testpackage = testpackage! {
    /// #     "__init__.py" => ""
    /// # };
    /// # let package_info = PackageInfo::build(testpackage.path()).unwrap();
    /// let children = package_info
    ///     .get_child_items(package_info.get_root().token())?
    ///     .collect::<Vec<PackageItem>>();
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_child_items(
        &self,
        token: PackageToken,
    ) -> Result<impl Iterator<Item = PackageItem>> {
        let package = self.get_package(token)?;

        let child_packages_iter = package
            .packages
            .iter()
            .map(|p| self.get_package(*p).unwrap())
            .map(PackageItem::Package);
        let child_modules_iter = package
            .modules
            .iter()
            .map(|m| self.get_module(*m).unwrap())
            .map(PackageItem::Module);
        let iter = child_packages_iter.chain(child_modules_iter);

        let v = iter.collect::<Vec<_>>();

        Ok(v.into_iter())
    }

    /// Get an iterator over the descendant items of the passed package.
    ///
    /// ```
    /// # use anyhow::Result;
    /// # use pyimports::{testpackage};
    /// # use pyimports::testutils::TestPackage;
    /// use pyimports::prelude::*;
    /// use pyimports::package_info::{PackageInfo,PackageItem};
    ///
    /// # fn main() -> Result<()> {
    /// # let testpackage = testpackage! {
    /// #     "__init__.py" => ""
    /// # };
    /// # let package_info = PackageInfo::build(testpackage.path()).unwrap();
    /// let descendants = package_info
    ///     .get_descendant_items(package_info.get_root().token())?
    ///     .collect::<Vec<PackageItem>>();
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_descendant_items(
        &self,
        token: PackageToken,
    ) -> Result<impl Iterator<Item = PackageItem>> {
        let children = self.get_child_items(token)?;
        let iter = children.chain(
            self.get_child_items(token)
                .unwrap()
                .filter_packages()
                .flat_map(|child_package| self.get_descendant_items(child_package.token).unwrap()),
        );
        let v = iter.collect::<Vec<_>>();
        Ok(v.into_iter())
    }

    /// Get an iterator over all the package items.
    pub fn get_all_items(&self) -> impl Iterator<Item = PackageItem> {
        let iter = std::iter::once(PackageItem::Package(self.get_root()))
            .chain(self.get_descendant_items(self.root).unwrap());
        let v = iter.collect::<Vec<_>>();
        v.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{testpackage, testutils::TestPackage};
    use anyhow::Result;
    use maplit::hashset;
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;

    fn create_testpackage() -> Result<TestPackage> {
        Ok(testpackage! {
            "__init__.py" => "",
            "main.py" => "",
            "colors/__init__.py" => "",
            "colors/red.py" => "",
            "food/__init__.py" => "",
            "food/pizza.py" => "",
            "food/fruit/__init__.py" => "",
            "food/fruit/apple.py" => "",
            "data.txt" => ""
        })
    }

    #[test]
    fn test_get_parent_item() -> Result<()> {
        let testpackage = create_testpackage()?;
        let package_info = PackageInfo::build(testpackage.path())?;

        let root_package = package_info.get_root();
        let colors_package = package_info
            .get_item_by_pypath("testpackage.colors")?
            .unwrap();
        let main = package_info
            .get_item_by_pypath("testpackage.main")?
            .unwrap();

        assert_eq!(
            package_info.get_parent_package(colors_package.token())?,
            Some(root_package)
        );

        assert_eq!(
            package_info.get_parent_package(main.token())?,
            Some(root_package)
        );

        assert_eq!(
            package_info.get_parent_package(root_package.token.into())?,
            None
        );

        Ok(())
    }

    #[test]
    fn test_get_child_items() -> Result<()> {
        let testpackage = create_testpackage()?;
        let package_info = PackageInfo::build(testpackage.path())?;

        assert_eq!(
            package_info
                .get_child_items(package_info.get_root().token)
                .unwrap()
                .map(|item| item.to_string())
                .collect::<HashSet<_>>(),
            hashset! {
                "Module(testpackage.__init__)".into(),
                "Module(testpackage.main)".into(),
                "Package(testpackage.colors)".into(),
                "Package(testpackage.food)".into(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_get_descendant_items() -> Result<()> {
        let testpackage = create_testpackage()?;
        let package_info = PackageInfo::build(testpackage.path())?;

        assert_eq!(
            package_info
                .get_descendant_items(package_info.get_root().token)
                .unwrap()
                .map(|item| item.to_string())
                .collect::<HashSet<_>>(),
            hashset! {
                "Module(testpackage.__init__)".into(),
                "Module(testpackage.main)".into(),
                //
                "Package(testpackage.colors)".into(),
                "Module(testpackage.colors.__init__)".into(),
                "Module(testpackage.colors.red)".into(),
                //
                "Package(testpackage.food)".into(),
                "Module(testpackage.food.__init__)".into(),
                "Module(testpackage.food.pizza)".into(),
                //
                "Package(testpackage.food.fruit)".into(),
                "Module(testpackage.food.fruit.__init__)".into(),
                "Module(testpackage.food.fruit.apple)".into(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_get_all_items() -> Result<()> {
        let testpackage = create_testpackage()?;
        let package_info = PackageInfo::build(testpackage.path())?;

        assert_eq!(
            package_info
                .get_all_items()
                .map(|item| item.to_string())
                .collect::<HashSet<_>>(),
            hashset! {
                "Package(testpackage)".into(),
                //
                "Module(testpackage.__init__)".into(),
                "Module(testpackage.main)".into(),
                //
                "Package(testpackage.colors)".into(),
                "Module(testpackage.colors.__init__)".into(),
                "Module(testpackage.colors.red)".into(),
                //
                "Package(testpackage.food)".into(),
                "Module(testpackage.food.__init__)".into(),
                "Module(testpackage.food.pizza)".into(),
                //
                "Package(testpackage.food.fruit)".into(),
                "Module(testpackage.food.fruit.__init__)".into(),
                "Module(testpackage.food.fruit.apple)".into(),
            }
        );

        Ok(())
    }
}
