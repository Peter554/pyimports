mod filesystem;

use anyhow::Result;
use slotmap::{new_key_type, SlotMap};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::utils::path_to_pypath;

new_key_type! { pub struct PackageToken; }
new_key_type! { pub struct ModuleToken; }

#[derive(Debug, Clone)]
pub struct Package {
    pub path: PathBuf,
    pub pypath: String,
    //
    pub token: PackageToken,
    pub parent: Option<PackageToken>,
    pub packages: HashSet<PackageToken>,
    pub modules: HashSet<ModuleToken>,
    pub init_module: Option<ModuleToken>,
}

impl Package {
    fn new(
        token: PackageToken,
        parent_token: Option<PackageToken>,
        path: &Path,
        root_path: &Path,
    ) -> Package {
        let pypath = path_to_pypath(path, root_path).unwrap();
        Package {
            token,
            parent: parent_token,
            packages: HashSet::new(),
            modules: HashSet::new(),
            init_module: None,
            pypath,
            path: path.to_path_buf(),
        }
    }

    fn _unit_test_string(&self) -> String {
        format!("package:{}", self.pypath)
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub path: PathBuf,
    pub pypath: String,
    pub is_init: bool,
    //
    pub token: ModuleToken,
    pub parent: PackageToken,
}

impl Module {
    fn new(
        token: ModuleToken,
        parent_token: PackageToken,
        path: &Path,
        root_path: &Path,
    ) -> Module {
        let pypath = &path_to_pypath(path, root_path).unwrap();
        Module {
            token,
            parent: parent_token,
            pypath: pypath.to_string(),
            path: path.to_path_buf(),
            is_init: path.file_name().unwrap().to_str().unwrap() == "__init__.py",
        }
    }

    fn _unit_test_string(&self) -> String {
        format!("module:{}", self.pypath)
    }
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    root: PackageToken,
    packages: SlotMap<PackageToken, Package>,
    modules: SlotMap<ModuleToken, Module>,
    packages_by_path: HashMap<PathBuf, PackageToken>,
    packages_by_pypath: HashMap<String, PackageToken>,
    modules_by_path: HashMap<PathBuf, ModuleToken>,
    modules_by_pypath: HashMap<String, ModuleToken>,
}

#[derive(Debug, Clone)]
pub enum PackageItem<'a> {
    Package(&'a Package),
    Module(&'a Module),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackageItemToken {
    Package(PackageToken),
    Module(ModuleToken),
}

impl From<PackageToken> for PackageItemToken {
    fn from(value: PackageToken) -> Self {
        PackageItemToken::Package(value)
    }
}

impl From<ModuleToken> for PackageItemToken {
    fn from(value: ModuleToken) -> Self {
        PackageItemToken::Module(value)
    }
}

impl<'a> PackageItem<'a> {
    pub fn token(&'a self) -> PackageItemToken {
        match self {
            PackageItem::Package(p) => p.token.into(),
            PackageItem::Module(m) => m.token.into(),
        }
    }
}

impl<'a> PackageInfo {
    pub fn build(root_path: &Path) -> Result<PackageInfo> {
        let mut packages = SlotMap::with_key();
        let mut modules = SlotMap::with_key();
        let mut packages_by_path = HashMap::new();
        let mut packages_by_pypath = HashMap::new();
        let mut modules_by_path = HashMap::new();
        let mut modules_by_pypath = HashMap::new();

        let root =
            packages.insert_with_key(|token| Package::new(token, None, root_path, root_path));
        packages_by_path.insert(root_path.to_path_buf(), root);
        packages_by_pypath.insert(path_to_pypath(root_path, root_path)?, root);

        let fs_items = filesystem::DirectoryReader::new()
            .exclude_hidden_items()
            .filter_file_extension("py")
            .read(root_path)?
            .into_iter()
            .skip(1); // Skip first item since this is the root, which we already have.

        for fs_item in fs_items {
            match fs_item {
                filesystem::FsItem::Directory { path } => {
                    let parent_token = packages_by_path.get(path.parent().unwrap()).unwrap();
                    let token = packages.insert_with_key(|token| {
                        Package::new(token, Some(*parent_token), &path, root_path)
                    });
                    let parent = packages.get_mut(*parent_token).unwrap();
                    parent.packages.insert(token);
                    packages_by_path.insert(path.clone(), token);
                    packages_by_pypath.insert(path_to_pypath(&path, root_path)?, token);
                }
                filesystem::FsItem::File { path } => {
                    let parent_token = packages_by_path.get(path.parent().unwrap()).unwrap();
                    let token = modules.insert_with_key(|token| {
                        Module::new(token, *parent_token, &path, root_path)
                    });
                    let is_init = modules.get(token).unwrap().is_init;
                    let parent = packages.get_mut(*parent_token).unwrap();
                    parent.modules.insert(token);
                    if is_init {
                        parent.init_module = Some(token);
                    }
                    modules_by_path.insert(path.clone(), token);
                    modules_by_pypath.insert(path_to_pypath(&path, root_path)?, token);
                }
            }
        }

        Ok(PackageInfo {
            root,
            packages,
            modules,
            packages_by_path,
            packages_by_pypath,
            modules_by_path,
            modules_by_pypath,
        })
    }

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
                        .filter_map(filter_packages)
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
            .chain(self.get_descendant_items(self.root).unwrap());
        let v = iter.collect::<Vec<_>>();
        v.into_iter()
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutils::TestPackage;
    use maplit::{hashmap, hashset};
    use pretty_assertions::assert_eq;

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
    fn test_build() -> Result<()> {
        let test_package = create_test_package()?;
        PackageInfo::build(test_package.path())?;
        Ok(())
    }

    #[test]
    fn test_get_child_items() -> Result<()> {
        let test_package = create_test_package()?;
        let package_info = PackageInfo::build(test_package.path())?;

        assert_eq!(
            package_info
                .get_child_items(package_info.root)
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
                .get_descendant_items(package_info.root)
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
