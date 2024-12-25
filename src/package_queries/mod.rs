use anyhow::Result;
use std::path::Path;

use crate::package_discovery::{
    Module, ModuleToken, Package, PackageInfo, PackageItem, PackageItemToken, PackageToken,
};

pub struct PackageQueries<'a> {
    p: &'a PackageInfo,
}

impl<'a> PackageQueries<'a> {
    pub fn new(package_info: &'a PackageInfo) -> Self {
        PackageQueries { p: package_info }
    }

    pub fn get_item_by_path(&self, path: &Path) -> Option<PackageItem> {
        if let Some(package) = self.p.packages_by_path.get(path) {
            self.get_package(*package).map(PackageItem::Package)
        } else if let Some(module) = self.p.modules_by_path.get(path) {
            self.get_module(*module).map(PackageItem::Module)
        } else {
            None
        }
    }

    pub fn get_item_by_pypath(&self, pypath: &str) -> Option<PackageItem> {
        if let Some(package) = self.p.packages_by_pypath.get(pypath) {
            self.get_package(*package).map(PackageItem::Package)
        } else if let Some(module) = self.p.modules_by_pypath.get(pypath) {
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
        self.p.packages.get(token)
    }

    pub fn get_module(&self, token: ModuleToken) -> Option<&Module> {
        self.p.modules.get(token)
    }

    pub fn get_root(&self) -> &Package {
        self.get_package(self.p.root).unwrap()
    }

    pub fn get_child_items(
        &'a self,
        token: PackageToken,
    ) -> Option<impl Iterator<Item = PackageItem<'a>>> {
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
        &'a self,
        token: PackageToken,
    ) -> Option<impl Iterator<Item = PackageItem<'a>>> {
        match self.get_child_items(token) {
            Some(children) => {
                let iter = children.chain(
                    self.get_child_items(token)
                        .unwrap()
                        .filter_map(PackageQueries::filter_packages)
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

    pub fn get_all_items(&'a self) -> impl Iterator<Item = PackageItem<'a>> {
        let iter = std::iter::once(PackageItem::Package(self.get_root()))
            .chain(self.get_descendant_items(self.p.root).unwrap());
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
    use std::collections::HashSet;

    use super::*;
    use crate::testutils::TestPackage;
    use maplit::{hashmap, hashset};
    use pretty_assertions::assert_eq;

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
        let test_package = TestPackage::new(
            "testpackage",
            hashmap! {
                "__init__.py" => "",
                "main.py" => "",
                "colors/__init__.py" => "",
                "colors/red.py" => "",
                "food/__init__.py" => "",
                "food/pizza.py" => "",
                "food/fruit/__init__.py" => "",
                "food/fruit/apple.py" => "",
                "data.txt" => "",
            },
        )?;
        Ok(test_package)
    }

    #[test]
    fn test_get_child_items() -> Result<()> {
        let test_package = create_test_package()?;
        let package_info = PackageInfo::build(test_package.path())?;
        let package_queries = package_info.queries();

        assert_eq!(
            package_queries
                .get_child_items(package_queries.get_root().token)
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
        let package_queries = package_info.queries();

        assert_eq!(
            package_queries
                .get_descendant_items(package_queries.get_root().token)
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
        let package_queries = package_info.queries();

        assert_eq!(
            package_queries
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
