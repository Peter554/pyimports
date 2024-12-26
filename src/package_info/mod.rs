mod filesystem;
mod queries;

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
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub(crate) root: PackageToken,
    pub(crate) packages: SlotMap<PackageToken, Package>,
    pub(crate) modules: SlotMap<ModuleToken, Module>,
    pub(crate) packages_by_path: HashMap<PathBuf, PackageToken>,
    pub(crate) packages_by_pypath: HashMap<String, PackageToken>,
    pub(crate) modules_by_path: HashMap<PathBuf, ModuleToken>,
    pub(crate) modules_by_pypath: HashMap<String, ModuleToken>,
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

impl PackageInfo {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutils::{testpackage, TestPackage};
    use maplit::hashset;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_build() -> Result<()> {
        let test_package = testpackage! {
            "__init__.py" => "",
            "main.py" => "",
            "colors/__init__.py" => "",
            "colors/red.py" => "",
            "data.txt" => ""
        };

        let package_info = PackageInfo::build(test_package.path())?;

        let root_package_token = *package_info.packages_by_pypath.get("testpackage").unwrap();
        let root_package_init_token = *package_info
            .modules_by_pypath
            .get("testpackage.__init__")
            .unwrap();
        let main_token = *package_info
            .modules_by_pypath
            .get("testpackage.main")
            .unwrap();
        let colors_package_token = *package_info
            .packages_by_pypath
            .get("testpackage.colors")
            .unwrap();
        let colors_package_init_token = *package_info
            .modules_by_pypath
            .get("testpackage.colors.__init__")
            .unwrap();
        let red_token = *package_info
            .modules_by_pypath
            .get("testpackage.colors.red")
            .unwrap();

        let root_package = package_info.packages.get(root_package_token).unwrap();
        assert_eq!(root_package.parent, None);
        assert_eq!(root_package.init_module, Some(root_package_init_token));
        assert_eq!(
            root_package.modules,
            hashset! {root_package_init_token, main_token}
        );
        assert_eq!(root_package.packages, hashset! {colors_package_token});

        let colors_package = package_info.packages.get(colors_package_token).unwrap();
        assert_eq!(colors_package.parent, Some(root_package_token));
        assert_eq!(colors_package.init_module, Some(colors_package_init_token));
        assert_eq!(
            colors_package.modules,
            hashset! {colors_package_init_token, red_token}
        );
        assert_eq!(colors_package.packages, hashset! {});

        let root_package_init = package_info.modules.get(root_package_init_token).unwrap();
        assert_eq!(root_package_init.is_init, true);
        assert_eq!(root_package_init.parent, root_package_token);

        let main = package_info.modules.get(main_token).unwrap();
        assert_eq!(main.is_init, false);
        assert_eq!(main.parent, root_package_token);

        Ok(())
    }
}