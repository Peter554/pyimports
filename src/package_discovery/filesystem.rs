use anyhow::Result;
use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub enum FsItem {
    Directory { path: PathBuf },
    File { path: PathBuf },
}

struct DirectoryFilter<'a> {
    f: Box<dyn Fn(&Path) -> bool + Sync + 'a>,
}

impl<'a> DirectoryFilter<'a> {
    fn filter(&'a self, path: &Path) -> bool {
        (self.f)(path)
    }
}

struct FileFilter<'a> {
    f: Box<dyn Fn(&Path) -> bool + Sync + 'a>,
}

impl<'a> FileFilter<'a> {
    fn filter(&'a self, path: &Path) -> bool {
        (self.f)(path)
    }
}

pub struct DirectoryReader<'a> {
    dir_filters: Vec<DirectoryFilter<'a>>,
    file_filters: Vec<FileFilter<'a>>,
}

impl Default for DirectoryReader<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> DirectoryReader<'a> {
    pub fn new() -> Self {
        DirectoryReader {
            dir_filters: vec![],
            file_filters: vec![],
        }
    }

    pub fn with_directory_filter<F>(mut self, f: F) -> Self
    where
        F: Fn(&Path) -> bool + Sync + 'a,
    {
        self.dir_filters.push(DirectoryFilter { f: Box::new(f) });
        self
    }

    pub fn with_file_filter<F>(mut self, f: F) -> Self
    where
        F: Fn(&Path) -> bool + Sync + 'a,
    {
        self.file_filters.push(FileFilter { f: Box::new(f) });
        self
    }

    pub fn exclude_hidden_items(self) -> Self {
        self.with_directory_filter(|path| {
            !path.file_name().unwrap().to_str().unwrap().starts_with(".")
        })
        .with_file_filter(|path| !path.file_name().unwrap().to_str().unwrap().starts_with("."))
    }

    pub fn filter_file_extension(self, extension: &'a str) -> Self {
        self.with_file_filter(move |path| {
            path.extension().unwrap_or_default().to_str().unwrap() == extension
        })
    }

    pub fn read(&'a self, path: &Path) -> Result<Vec<FsItem>> {
        if !self.dir_filters.iter().all(|f| f.filter(path)) {
            return Ok(vec![]);
        }

        let mut v = vec![FsItem::Directory {
            path: path.to_path_buf(),
        }];

        v.extend(
            fs::read_dir(path)?
                .par_bridge()
                .try_fold(std::vec::Vec::new, |mut v, dir_item| -> Result<_> {
                    let dir_item = dir_item?;
                    let path = dir_item.path();
                    let file_type = dir_item.file_type()?;
                    let is_dir = file_type.is_dir();
                    let is_file = file_type.is_file();
                    let is_symlink = file_type.is_symlink();
                    if is_dir {
                        v.extend(self.read(&path)?);
                    } else if is_file && self.file_filters.iter().all(|filter| filter.filter(&path))
                    {
                        v.push(FsItem::File { path: path.clone() });
                    }
                    Ok(v)
                })
                .try_reduce(std::vec::Vec::new, |mut v, fs_items| {
                    v.extend(fs_items);
                    Ok(v)
                })?,
        );

        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutils::TestPackage;
    use maplit::{hashmap, hashset};
    use std::collections::HashSet;

    fn create_test_package() -> Result<TestPackage> {
        let test_package = TestPackage::new(
            "testpackage",
            hashmap! {
                "__init__.py" => "",
                "main.py" => "",
                "food/__init__.py" => "",
                "food/pizza.py" => "",
                "food/fruit/__init__.py" => "",
                "food/fruit/apple.py" => "",
                "foo.txt" => "",
                ".gitignore" => "",
                ".linter/config" => ""
            },
        )?;
        Ok(test_package)
    }

    #[test]
    fn test_build() -> Result<()> {
        let test_package = create_test_package()?;

        let paths = DirectoryReader::new().read(test_package.path())?;

        assert_eq!(paths.len(), 13);
        assert_eq!(
            paths
                .into_iter()
                .map(|p| {
                    match p {
                        FsItem::Directory { path } => path,
                        FsItem::File { path } => path,
                    }
                })
                .collect::<HashSet<_>>(),
            hashset![
                test_package.path().to_path_buf(),
                test_package.path().join("__init__.py"),
                test_package.path().join("main.py"),
                //
                test_package.path().join("food"),
                test_package.path().join("food/__init__.py"),
                test_package.path().join("food/pizza.py"),
                //
                test_package.path().join("food/fruit"),
                test_package.path().join("food/fruit/__init__.py"),
                test_package.path().join("food/fruit/apple.py"),
                //
                test_package.path().join("foo.txt"),
                test_package.path().join(".gitignore"),
                test_package.path().join(".linter"),
                test_package.path().join(".linter/config"),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_build_with_filters() -> Result<()> {
        let test_package = create_test_package()?;

        let paths = DirectoryReader::new()
            .exclude_hidden_items()
            .filter_file_extension("py")
            .read(test_package.path())?;

        assert_eq!(paths.len(), 9);
        assert_eq!(
            paths
                .into_iter()
                .map(|p| {
                    match p {
                        FsItem::Directory { path } => path,
                        FsItem::File { path } => path,
                    }
                })
                .collect::<HashSet<_>>(),
            hashset![
                test_package.path().to_path_buf(),
                test_package.path().join("__init__.py"),
                test_package.path().join("main.py"),
                //
                test_package.path().join("food"),
                test_package.path().join("food/__init__.py"),
                test_package.path().join("food/pizza.py"),
                //
                test_package.path().join("food/fruit"),
                test_package.path().join("food/fruit/__init__.py"),
                test_package.path().join("food/fruit/apple.py"),
            ]
        );

        Ok(())
    }
}
