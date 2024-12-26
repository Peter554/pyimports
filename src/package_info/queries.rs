use std::{collections::HashSet, path::Path};

use maplit::hashset;

use crate::package_info::{
    Module, ModuleToken, Package, PackageInfo, PackageItem, PackageItemToken, PackageToken,
};

#[derive(Debug)]
pub(crate) struct PackageContents {
    pub root: PackageItemToken,
    pub descendant_packages: HashSet<PackageItemToken>,
    pub descendant_modules: HashSet<PackageItemToken>,
    pub descendant_items: HashSet<PackageItemToken>,
    pub all_items: HashSet<PackageItemToken>,
}

impl PackageInfo {
    pub fn get_item_by_path(&self, path: &Path) -> Option<PackageItem> {
        if let Some(package) = self.packages_by_path.get(path) {
            self.get_package(*package).map(PackageItem::Package)
        } else if let Some(module) = self.modules_by_path.get(path) {
            self.get_module(*module).map(PackageItem::Module)
        } else {
            None
        }
    }

    pub fn get_item_by_pypath(&self, pypath: &str) -> Option<PackageItem> {
        if let Some(package) = self.packages_by_pypath.get(pypath) {
            self.get_package(*package).map(PackageItem::Package)
        } else if let Some(module) = self.modules_by_pypath.get(pypath) {
            self.get_module(*module).map(PackageItem::Module)
        } else {
            None
        }
    }

    pub fn get_item(&self, token: PackageItemToken) -> Option<PackageItem> {
        match token {
            PackageItemToken::Package(token) => self.get_package(token).map(PackageItem::Package),
            PackageItemToken::Module(token) => self.get_module(token).map(PackageItem::Module),
        }
    }

    pub fn get_package(&self, token: PackageToken) -> Option<&Package> {
        self.packages.get(token)
    }

    pub fn get_module(&self, token: ModuleToken) -> Option<&Module> {
        self.modules.get(token)
    }

    pub fn get_root(&self) -> &Package {
        self.get_package(self.root).unwrap()
    }

    pub fn get_child_items(
        &self,
        token: PackageToken,
    ) -> Option<impl Iterator<Item = PackageItem>> {
        match self.get_package(token) {
            Some(package) => {
                let child_packages_iter = package
                    .packages
                    .iter()
                    .filter_map(|p| self.get_package(*p))
                    .map(PackageItem::Package);
                let child_modules_iter = package
                    .modules
                    .iter()
                    .filter_map(|m| self.get_module(*m))
                    .map(PackageItem::Module);
                let v = child_packages_iter
                    .chain(child_modules_iter)
                    .collect::<Vec<_>>();
                Some(v.into_iter())
            }
            None => None,
        }
    }

    pub fn get_descendant_items(
        &self,
        token: PackageToken,
    ) -> Option<impl Iterator<Item = PackageItem>> {
        match self.get_child_items(token) {
            Some(children) => {
                let iter = children.chain(
                    self.get_child_items(token)
                        .unwrap()
                        .filter_map(PackageInfo::filter_packages)
                        .flat_map(|child_package| {
                            self.get_descendant_items(child_package.token).unwrap()
                        }),
                );
                let v = iter.collect::<Vec<_>>();
                Some(v.into_iter())
            }
            None => None,
        }
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

    pub(crate) fn get_package_contents(&self, package: PackageToken) -> PackageContents {
        let descendant_packages = self
            .get_descendant_items(package)
            .unwrap()
            .filter_map(PackageInfo::filter_packages)
            .map(|o| o.token.into())
            .collect::<HashSet<_>>();

        let descendant_modules = self
            .get_descendant_items(package)
            .unwrap()
            .filter_map(PackageInfo::filter_modules)
            .map(|o| o.token.into())
            .collect::<HashSet<_>>();

        let descendant_items = &descendant_packages | &descendant_modules;
        let all_items = &hashset! {package.into()} | &descendant_items;

        PackageContents {
            root: package.into(),
            descendant_packages,
            descendant_modules,
            descendant_items,
            all_items,
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
