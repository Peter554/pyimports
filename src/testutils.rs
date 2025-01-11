use crate::imports_info::ImportsInfo;
use crate::package_info::{PackageInfo, PackageItemToken};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempdir::TempDir;

#[doc(hidden)]
pub struct TestPackage {
    // Need to move the TempDir into TestPackage to avoid it being dropped.
    _temp_dir: TempDir,
    dir_path: PathBuf,
}

impl TestPackage {
    pub fn new(name: &str, modules: HashMap<&str, &str>) -> Result<TestPackage> {
        let temp_dir = TempDir::new("")?;
        let dir_path = temp_dir.path().join(name);
        fs::create_dir(&dir_path)?;
        let testpackage = TestPackage {
            _temp_dir: temp_dir,
            dir_path,
        };
        for (path, contents) in modules.into_iter() {
            testpackage.add_file(path, contents)?;
        }
        Ok(testpackage)
    }

    pub fn path(&self) -> &Path {
        &self.dir_path
    }

    pub fn add_file(&self, path: &str, contents: &str) -> Result<()> {
        let file_path = self.dir_path.join(path);
        let file_dir = file_path.parent().unwrap();
        fs::create_dir_all(file_dir)?;
        let mut tmp_file = File::create(file_path)?;
        write!(tmp_file, "{}", contents)?;
        Ok(())
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! testpackage {
    ($($k:expr => $v:expr),*) => {{
        let testpackage = TestPackage::new("testpackage", std::collections::HashMap::new())?;
        $(
            testpackage.add_file($k, $v)?;
        )*
        testpackage
    }};
}

impl PackageInfo {
    pub(crate) fn _item(&self, pypath: &str) -> PackageItemToken {
        self.get_item_by_pypath(&pypath.parse().unwrap())
            .unwrap()
            .token()
    }
}

impl ImportsInfo {
    pub(crate) fn _item(&self, pypath: &str) -> PackageItemToken {
        self.package_info()._item(pypath)
    }
}
