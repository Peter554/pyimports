use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempdir::TempDir;

pub(crate) struct TestPackage {
    temp_dir: TempDir,
}

impl TestPackage {
    pub fn new(modules: HashMap<&str, &str>) -> Result<TestPackage> {
        let test_package = TestPackage {
            temp_dir: TempDir::new("")?,
        };
        for (module, contents) in modules.into_iter() {
            let path = module.replace(".", "/") + ".py";
            test_package.add_file(&path, contents)?;
        }
        Ok(test_package)
    }

    pub fn path(&self) -> &Path {
        return self.temp_dir.path();
    }

    fn add_file(&self, path: &str, contents: &str) -> Result<()> {
        let file_path = self.temp_dir.path().join(path);
        let file_dir = file_path.parent().unwrap();
        fs::create_dir_all(file_dir)?;
        let mut tmp_file = File::create(file_path)?;
        write!(tmp_file, "{}", contents)?;
        Ok(())
    }
}
