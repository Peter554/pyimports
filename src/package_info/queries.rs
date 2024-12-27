use anyhow::Result;
use std::path::Path;

use crate::package_info::{
    Module, ModuleToken, Package, PackageInfo, PackageItem, PackageItemToken, PackageToken,
};
use crate::Error;

impl PackageInfo {
    pub fn get_item_by_path(&self, path: &Path) -> Option<PackageItem> {
        if let Some(package) = self.packages_by_path.get(path) {
            Some(self.get_package(*package).unwrap().into())
        } else {
            self.modules_by_path
                .get(path)
                .map(|module| self.get_module(*module).unwrap().into())
        }
    }

    pub fn get_item_by_pypath(&self, pypath: &str) -> Option<PackageItem> {
        if let Some(package) = self.packages_by_pypath.get(pypath) {
            Some(self.get_package(*package).unwrap().into())
        } else {
            self.modules_by_pypath
                .get(pypath)
                .map(|module| self.get_module(*module).unwrap().into())
        }
    }

    pub fn get_item(&self, token: PackageItemToken) -> Result<PackageItem> {
        match token {
            PackageItemToken::Package(token) => Ok(self.get_package(token)?.into()),
            PackageItemToken::Module(token) => Ok(self.get_module(token)?.into()),
        }
    }

    pub fn get_package(&self, token: PackageToken) -> Result<&Package> {
        match self.packages.get(token) {
            Some(package) => Ok(package),
            None => Err(Error::UnknownPackage(token))?,
        }
    }

    pub fn get_module(&self, token: ModuleToken) -> Result<&Module> {
        match self.modules.get(token) {
            Some(module) => Ok(module),
            None => Err(Error::UnknownModule(token))?,
        }
    }

    pub fn get_root(&self) -> &Package {
        self.get_package(self.root).unwrap()
    }

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

    pub fn get_descendant_items(
        &self,
        token: PackageToken,
    ) -> Result<impl Iterator<Item = PackageItem>> {
        let children = self.get_child_items(token)?;
        let iter = children.chain(
            self.get_child_items(token)
                .unwrap()
                .filter_map(PackageInfo::filter_packages)
                .flat_map(|child_package| self.get_descendant_items(child_package.token).unwrap()),
        );
        let v = iter.collect::<Vec<_>>();
        Ok(v.into_iter())
    }

    pub fn get_all_items(&self) -> impl Iterator<Item = PackageItem> {
        let iter = std::iter::once(PackageItem::Package(self.get_root()))
            .chain(self.get_descendant_items(self.root).unwrap());
        let v = iter.collect::<Vec<_>>();
        v.into_iter()
    }

    pub fn filter_packages(item: PackageItem<'_>) -> Option<&Package> {
        match item {
            PackageItem::Package(package) => Some(package),
            _ => None,
        }
    }

    pub fn filter_modules(item: PackageItem<'_>) -> Option<&Module> {
        match item {
            PackageItem::Module(module) => Some(module),
            _ => None,
        }
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

    impl Package {
        fn _unit_test_string(&self) -> String {
            format!("package:{}", self.pypath)
        }
    }

    impl Module {
        fn _unit_test_string(&self) -> String {
            format!("module:{}", self.pypath)
        }
    }

    fn create_test_package() -> Result<TestPackage> {
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
    fn test_get_child_items() -> Result<()> {
        let test_package = create_test_package()?;
        let package_info = PackageInfo::build(test_package.path())?;

        assert_eq!(
            package_info
                .get_child_items(package_info.get_root().token)
                .unwrap()
                .map(|item| {
                    match item {
                        PackageItem::Package(p) => p._unit_test_string(),
                        PackageItem::Module(m) => m._unit_test_string(),
                    }
                })
                .collect::<HashSet<_>>(),
            hashset! {
                "module:testpackage.__init__".into(),
                "module:testpackage.main".into(),
                "package:testpackage.colors".into(),
                "package:testpackage.food".into(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_get_descendant_items() -> Result<()> {
        let test_package = create_test_package()?;
        let package_info = PackageInfo::build(test_package.path())?;

        assert_eq!(
            package_info
                .get_descendant_items(package_info.get_root().token)
                .unwrap()
                .map(|item| {
                    match item {
                        PackageItem::Package(p) => p._unit_test_string(),
                        PackageItem::Module(m) => m._unit_test_string(),
                    }
                })
                .collect::<HashSet<_>>(),
            hashset! {
                "module:testpackage.__init__".into(),
                "module:testpackage.main".into(),
                //
                "package:testpackage.colors".into(),
                "module:testpackage.colors.__init__".into(),
                "module:testpackage.colors.red".into(),
                //
                "package:testpackage.food".into(),
                "module:testpackage.food.__init__".into(),
                "module:testpackage.food.pizza".into(),
                //
                "package:testpackage.food.fruit".into(),
                "module:testpackage.food.fruit.__init__".into(),
                "module:testpackage.food.fruit.apple".into(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_get_all_items() -> Result<()> {
        let test_package = create_test_package()?;
        let package_info = PackageInfo::build(test_package.path())?;

        assert_eq!(
            package_info
                .get_all_items()
                .map(|item| {
                    match item {
                        PackageItem::Package(p) => p._unit_test_string(),
                        PackageItem::Module(m) => m._unit_test_string(),
                    }
                })
                .collect::<HashSet<_>>(),
            hashset! {
                "package:testpackage".into(),
                //
                "module:testpackage.__init__".into(),
                "module:testpackage.main".into(),
                //
                "package:testpackage.colors".into(),
                "module:testpackage.colors.__init__".into(),
                "module:testpackage.colors.red".into(),
                //
                "package:testpackage.food".into(),
                "module:testpackage.food.__init__".into(),
                "module:testpackage.food.pizza".into(),
                //
                "package:testpackage.food.fruit".into(),
                "module:testpackage.food.fruit.__init__".into(),
                "module:testpackage.food.fruit.apple".into(),
            }
        );

        Ok(())
    }
}
