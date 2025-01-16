//! The `package_info` module provides a rich representation of a python package.
//! See [`PackageInfo`].

mod filesystem;
mod queries;

#[doc(hidden)]
#[cfg(feature = "grimp_compare")]
pub(crate) mod grimp_compare;

use crate::errors::Error;
use crate::pypath::Pypath;
use anyhow::Result;
use core::fmt;
use derive_more::{IsVariant, Unwrap};
use getset::{CopyGetters, Getters};
use maplit::hashset;
pub use queries::{MutPackageItemIterator, PackageItemIterator};
use slotmap::{new_key_type, SlotMap};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

new_key_type! {
    /// A token used to identify an item within a python package.
    pub struct PackageItemToken;
}

/// A unified representation of an item within a package.
///
/// ```
/// # use std::collections::HashSet;
/// # use anyhow::Result;
/// # use pyimports::{testpackage,testutils::TestPackage};
/// use pyimports::package_info::{PackageInfo,Package,Module,PackageItem};
///
/// # fn main() -> Result<()> {
/// let testpackage = testpackage! {
///     "__init__.py" => ""
/// };
///
/// let package_info = PackageInfo::build(testpackage.path())?;
///
/// let root_pkg: &PackageItem = package_info.get_item_by_pypath(&"testpackage".parse()?).unwrap();
/// let root_init: &PackageItem = package_info.get_item_by_pypath(&"testpackage.__init__".parse()?).unwrap();
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, IsVariant, Unwrap)]
#[unwrap(ref, ref_mut)]
pub enum PackageItem {
    /// A package.
    Package(Package),
    /// A module.
    Module(Module),
}

impl fmt::Display for PackageItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageItem::Package(p) => p.fmt(f),
            PackageItem::Module(m) => m.fmt(f),
        }
    }
}

impl From<Package> for PackageItem {
    fn from(value: Package) -> Self {
        PackageItem::Package(value)
    }
}

impl From<Module> for PackageItem {
    fn from(value: Module) -> Self {
        PackageItem::Module(value)
    }
}

impl TryFrom<PackageItem> for Package {
    type Error = Error;

    fn try_from(value: PackageItem) -> std::result::Result<Self, Self::Error> {
        match value {
            PackageItem::Package(package) => Ok(package),
            PackageItem::Module(_) => Err(Error::NotAPackage),
        }
    }
}

impl TryFrom<PackageItem> for Module {
    type Error = Error;

    fn try_from(value: PackageItem) -> std::result::Result<Self, Self::Error> {
        match value {
            PackageItem::Package(_) => Err(Error::NotAModule),
            PackageItem::Module(module) => Ok(module),
        }
    }
}

impl<'a> TryFrom<&'a PackageItem> for &'a Package {
    type Error = Error;

    fn try_from(value: &'a PackageItem) -> std::result::Result<Self, Self::Error> {
        match value {
            PackageItem::Package(package) => Ok(package),
            PackageItem::Module(_) => Err(Error::NotAPackage),
        }
    }
}

impl<'a> TryFrom<&'a PackageItem> for &'a Module {
    type Error = Error;

    fn try_from(value: &'a PackageItem) -> std::result::Result<Self, Self::Error> {
        match value {
            PackageItem::Package(_) => Err(Error::NotAModule),
            PackageItem::Module(module) => Ok(module),
        }
    }
}

impl PackageItem {
    /// The token for this package item.
    pub fn token(&self) -> PackageItemToken {
        match self {
            PackageItem::Package(p) => p.token,
            PackageItem::Module(m) => m.token,
        }
    }

    /// The filesystem path for this package item.
    pub fn path(&self) -> &Path {
        match self {
            PackageItem::Package(p) => &p.path,
            PackageItem::Module(m) => &m.path,
        }
    }

    /// The pypath for this package item.
    pub fn pypath(&self) -> &Pypath {
        match self {
            PackageItem::Package(p) => &p.pypath,
            PackageItem::Module(m) => &m.pypath,
        }
    }
}

/// A python package.
/// See also [`PackageItem`].
#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct Package {
    /// The absolute filesystem path to this package.
    #[getset(get = "pub")]
    path: PathBuf,
    /// The absolute pypath to this package.
    #[getset(get = "pub")]
    pypath: Pypath,

    /// This package.
    #[getset(get_copy = "pub")]
    token: PackageItemToken,
    /// The parent package.
    #[getset(get_copy = "pub")]
    parent: Option<PackageItemToken>,
    /// Child packages.
    #[getset(get = "pub")]
    packages: HashSet<PackageItemToken>,
    /// Child modules.
    #[getset(get = "pub")]
    modules: HashSet<PackageItemToken>,
    /// The init module.
    #[getset(get_copy = "pub")]
    init_module: Option<PackageItemToken>,
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Package({})", self.pypath)
    }
}

impl Package {
    fn new(
        token: PackageItemToken,
        parent_token: Option<PackageItemToken>,
        path: &Path,
        root_path: &Path,
    ) -> Package {
        let pypath = Pypath::from_path(path, root_path).unwrap();
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

/// A python module.
/// See also [`PackageItem`].
#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct Module {
    /// The absolute filesystem path to this module.
    #[getset(get = "pub")]
    path: PathBuf,
    /// The absolute pypath to this module.
    #[getset(get = "pub")]
    pypath: Pypath,
    /// True if this is an init module.
    #[getset(get_copy = "pub")]
    is_init: bool,

    /// This module.
    #[getset(get_copy = "pub")]
    token: PackageItemToken,
    /// The parent package.
    #[getset(get_copy = "pub")]
    parent: PackageItemToken,
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Module({})", self.pypath)
    }
}

impl Module {
    fn new(
        token: PackageItemToken,
        parent_token: PackageItemToken,
        path: &Path,
        root_path: &Path,
    ) -> Module {
        let pypath = Pypath::from_path(path, root_path).unwrap();
        Module {
            token,
            parent: parent_token,
            pypath,
            path: path.to_path_buf(),
            is_init: path.file_name().unwrap().to_str().unwrap() == "__init__.py",
        }
    }
}

/// A rich representation of a python package.
///
/// ```
/// # use std::collections::HashSet;
/// # use anyhow::Result;
/// # use pyimports::{testpackage,testutils::TestPackage};
/// use pyimports::package_info::PackageInfo;
///
/// # fn main() -> Result<()> {
/// let testpackage = testpackage! {
///     "__init__.py" => "",
///     "a.py" => "",
///     "b/__init__.py" => "",
///     "b/c.py" => ""
/// };
///
/// let package_info = PackageInfo::build(testpackage.path())?;
///
/// let all_items = package_info
///     .get_all_items()
///     .map(|item| item.to_string())
///     .collect::<HashSet<_>>();
///
/// assert_eq!(
///     all_items,
///     HashSet::from([
///         "Package(testpackage)".into(),
///         "Module(testpackage.__init__)".into(),
///         "Module(testpackage.a)".into(),
///         "Package(testpackage.b)".into(),
///         "Module(testpackage.b.__init__)".into(),
///         "Module(testpackage.b.c)".into(),
///     ])
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct PackageInfo {
    root: PackageItemToken,
    items: SlotMap<PackageItemToken, PackageItem>,
    items_by_path: HashMap<PathBuf, PackageItemToken>,
    items_by_pypath: HashMap<Pypath, PackageItemToken>,
}

impl PackageInfo {
    /// Builds [`PackageInfo`] from the passed filesystem path.
    /// The passed filesystem path should be the path to the root package.
    ///
    /// ```
    /// # use anyhow::Result;
    /// # use pyimports::{testpackage,testutils::TestPackage};
    /// use pyimports::package_info::PackageInfo;
    ///
    /// # fn main() -> Result<()> {
    /// let testpackage = testpackage! {
    ///     "__init__.py" => ""
    /// };
    ///
    /// let result = PackageInfo::build(testpackage.path());
    /// assert!(result.is_ok());
    /// # Ok(())
    /// # }
    /// ```
    pub fn build<T: AsRef<Path>>(root_path: T) -> Result<PackageInfo> {
        let root_path = root_path.as_ref();

        let mut items: SlotMap<PackageItemToken, PackageItem> = SlotMap::with_key();
        let mut items_by_path = HashMap::new();
        let mut items_by_pypath = HashMap::new();

        let root =
            items.insert_with_key(|token| Package::new(token, None, root_path, root_path).into());
        items_by_path.insert(root_path.to_path_buf(), root);
        items_by_pypath.insert(Pypath::from_path(root_path, root_path)?, root);

        let fs_items = filesystem::DirectoryReader::new()
            .with_hidden_items_excluded()
            .with_file_extension_filter("py")
            .read(root_path)?
            .skip(1); // Skip first item since this is the root, which we already have.

        for fs_item in fs_items {
            match fs_item {
                filesystem::FsItem::Directory { path } => {
                    let parent_token = items_by_path.get(path.parent().unwrap()).unwrap();
                    let token = items.insert_with_key(|token| {
                        Package::new(token, Some(*parent_token), &path, root_path).into()
                    });
                    let parent = items.get_mut(*parent_token).unwrap().unwrap_package_mut();
                    parent.packages.insert(token);
                    items_by_path.insert(path.clone(), token);
                    items_by_pypath.insert(Pypath::from_path(&path, root_path)?, token);
                }
                filesystem::FsItem::File { path } => {
                    let parent_token = items_by_path.get(path.parent().unwrap()).unwrap();
                    let token = items.insert_with_key(|token| {
                        Module::new(token, *parent_token, &path, root_path).into()
                    });
                    let is_init = items.get(token).unwrap().unwrap_module_ref().is_init;
                    let parent = items.get_mut(*parent_token).unwrap().unwrap_package_mut();
                    parent.modules.insert(token);
                    if is_init {
                        parent.init_module = Some(token);
                    }
                    items_by_path.insert(path.clone(), token);
                    items_by_pypath.insert(Pypath::from_path(&path, root_path)?, token);
                }
            }
        }

        Ok(PackageInfo {
            root,
            items,
            items_by_path,
            items_by_pypath,
        })
    }
}

impl From<PackageItemToken> for HashSet<PackageItemToken> {
    fn from(value: PackageItemToken) -> Self {
        hashset! { value }
    }
}

/// Extends a collection of package item tokens with all descendant items.
///
/// ```
/// # use anyhow::Result;
/// # use maplit::hashset;
/// # use pyimports::{testpackage,testutils::TestPackage};
/// use pyimports::prelude::*;
/// use pyimports::package_info::PackageInfo;
///
/// # fn main() -> Result<()> {
/// let testpackage = testpackage! {
///         "a.py" => "",
///         "b/c.py" => ""
///  };
///
/// let package_info = PackageInfo::build(testpackage.path())?;
///
/// let root = package_info.get_item_by_pypath(&"testpackage".parse()?).unwrap().token();
/// let a = package_info.get_item_by_pypath(&"testpackage.a".parse()?).unwrap().token();
/// let b = package_info.get_item_by_pypath(&"testpackage.b".parse()?).unwrap().token();
/// let c = package_info.get_item_by_pypath(&"testpackage.b.c".parse()?).unwrap().token();
///
/// let package_item_tokens = hashset! {root};
/// assert_eq!(
///     package_item_tokens.with_descendants(&package_info),
///     hashset! {root, a, b, c}
/// );
/// # Ok(())
/// # }
/// ```
pub trait ExtendWithDescendants:
    Sized + Clone + IntoIterator<Item = PackageItemToken> + Extend<PackageItemToken>
{
    /// Extend this collection of package item tokens with all descendant items.
    fn extend_with_descendants(&mut self, package_info: &PackageInfo) {
        for item in self.clone().into_iter() {
            let descendants = package_info
                .get_descendant_items(item)
                .unwrap()
                .map(|item| item.token());
            self.extend(descendants);
        }
    }

    /// Extend this collection of package item tokens with all descendant items.
    fn with_descendants(mut self, package_info: &PackageInfo) -> Self {
        self.extend_with_descendants(package_info);
        self
    }
}

impl<T: Sized + Clone + IntoIterator<Item = PackageItemToken> + Extend<PackageItemToken>>
    ExtendWithDescendants for T
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{testpackage, testutils::TestPackage};
    use maplit::hashset;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_build() -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => "",
            "main.py" => "",
            "colors/__init__.py" => "",
            "colors/red.py" => "",
            "data.txt" => ""
        };

        let package_info = PackageInfo::build(testpackage.path())?;

        let root_package_token = *package_info
            .items_by_pypath
            .get(&"testpackage".parse()?)
            .unwrap();
        let root_package_init_token = *package_info
            .items_by_pypath
            .get(&"testpackage.__init__".parse()?)
            .unwrap();
        let main_token = *package_info
            .items_by_pypath
            .get(&"testpackage.main".parse()?)
            .unwrap();
        let colors_package_token = *package_info
            .items_by_pypath
            .get(&"testpackage.colors".parse()?)
            .unwrap();
        let colors_package_init_token = *package_info
            .items_by_pypath
            .get(&"testpackage.colors.__init__".parse()?)
            .unwrap();
        let red_token = *package_info
            .items_by_pypath
            .get(&"testpackage.colors.red".parse()?)
            .unwrap();

        let root_package = package_info
            .items
            .get(root_package_token)
            .unwrap()
            .unwrap_package_ref();
        assert_eq!(root_package.parent, None);
        assert_eq!(root_package.init_module, Some(root_package_init_token));
        assert_eq!(
            root_package.modules,
            hashset! {root_package_init_token, main_token}
        );
        assert_eq!(root_package.packages, hashset! {colors_package_token});

        let colors_package = package_info
            .items
            .get(colors_package_token)
            .unwrap()
            .unwrap_package_ref();
        assert_eq!(colors_package.parent, Some(root_package_token));
        assert_eq!(colors_package.init_module, Some(colors_package_init_token));
        assert_eq!(
            colors_package.modules,
            hashset! {colors_package_init_token, red_token}
        );
        assert_eq!(colors_package.packages, hashset! {});

        let root_package_init = package_info
            .items
            .get(root_package_init_token)
            .unwrap()
            .unwrap_module_ref();
        assert_eq!(root_package_init.is_init, true);
        assert_eq!(root_package_init.parent, root_package_token);

        let main = package_info
            .items
            .get(main_token)
            .unwrap()
            .unwrap_module_ref();
        assert_eq!(main.is_init, false);
        assert_eq!(main.parent, root_package_token);

        Ok(())
    }
}
