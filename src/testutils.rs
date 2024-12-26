use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempdir::TempDir;

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
        let test_package = TestPackage {
            _temp_dir: temp_dir,
            dir_path,
        };
        for (path, contents) in modules.into_iter() {
            test_package.add_file(path, contents)?;
        }
        Ok(test_package)
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

/// A utility to create a test package.
///
/// This should probably be behind a `#[cfg(test)]`, but the use within
/// doctests seems to prevents this.
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
