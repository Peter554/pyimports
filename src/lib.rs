use anyhow::Result;

fn foo() -> Result<u8> {
    Ok(42)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use tempdir::TempDir;

    use super::*;

    struct TestPackage {
        temp_dir: TempDir,
    }

    impl TestPackage {
        fn new() -> Result<TestPackage> {
            Ok(TestPackage {
                temp_dir: TempDir::new("")?,
            })
        }

        fn path(&self) -> &Path {
            return self.temp_dir.path();
        }

        fn add_file(self, path: &str, contents: &str) -> Result<TestPackage> {
            let file_path = self.temp_dir.path().join(path);
            let file_dir = file_path.parent().unwrap();
            fs::create_dir_all(file_dir)?;
            let mut tmp_file = File::create(file_path)?;
            write!(tmp_file, "{}", contents)?;
            Ok(self)
        }
    }

    #[test]
    fn test_foo() -> Result<()> {
        let test_package = TestPackage::new()?
            .add_file(
                "__init__.py",
                "from a import b",
            )?
            .add_file(
                "__init__.py",
                "from a import b",
            )?;

        println!("{:?}", test_package.path());

        assert_eq!(foo().unwrap(), 42);
        Ok(())
    }
}
