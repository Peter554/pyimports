use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempdir::TempDir;

use crate::{ImportsInfo, PackageInfo, PackageItemToken};

#[doc(hidden)]
pub struct TestPackage {
    // Need to move the TempDir into TestPackage to avoid it being dropped.
    _temp_dir: TempDir,
    dir_path: PathBuf,
}

impl TestPackage {
    #[allow(missing_docs)]
    pub fn new(name: &str, modules: HashMap<&str, &str>) -> Result<TestPackage> {
        let temp_dir = TempDir::new("")?;
        let dir_path = temp_dir.path().join(name);
        fs::create_dir(&dir_path)?;
        let test_package = TestPackage {
            _temp_dir: temp_dir,
            dir_path,
        };
        for (path, contents) in modules.into_iter() {
            test_package.add_file(path, contents)?;
        }
        Ok(test_package)
    }

    #[allow(missing_docs)]
    pub fn path(&self) -> &Path {
        &self.dir_path
    }

    #[allow(missing_docs)]
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
        let test_package = TestPackage::new("testpackage", std::collections::HashMap::new())?;
        $(
            test_package.add_file($k, $v)?;
        )*
        test_package
    }};
}

impl PackageInfo {
    pub(crate) fn _item(&self, pypath: &str) -> PackageItemToken {
        self.get_item_by_pypath(pypath).unwrap().unwrap().token()
    }
}

impl ImportsInfo {
    pub(crate) fn _item(&self, pypath: &str) -> PackageItemToken {
        self.package_info()._item(pypath)
    }
}
