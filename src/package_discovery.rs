use anyhow::Result;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::{fs, io, path::Path};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PackageDiscoveryError {
    #[error("cannot read directory")]
    CannotReadDir(#[source] io::Error),

    #[error("cannot read directory entry")]
    CannotReadDirEntry(#[source] io::Error),

    #[error("cannot determine file type")]
    CannotDetermineFileType(#[source] io::Error),

    #[error("not a python package")]
    NotAPythonPackage,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Package {
    pub pypath: String,
    pub path: PathBuf,
    pub children: Vec<Arc<Package>>,
    pub modules: Vec<Arc<Module>>,
}

impl Package {
    pub fn new(pypath: String, path: PathBuf) -> Self {
        Package {
            pypath,
            path,
            children: vec![],
            modules: vec![],
        }
    }

    pub fn add_child(&mut self, child: Arc<Package>) {
        self.children.push(child);
    }

    pub fn add_module(&mut self, module: Arc<Module>) {
        self.modules.push(module);
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Module {
    pub pypath: String,
    pub path: PathBuf,
}

impl Module {
    pub fn new(pypath: String, path: PathBuf) -> Self {
        Module { pypath, path }
    }
}

pub fn discover_package(package_path: &Path) -> Result<Arc<Package>> {
    _discover_package(package_path, package_path)
}

fn _discover_package(root_package_path: &Path, package_path: &Path) -> Result<Arc<Package>> {
    let (is_package, files, dirs) = fs::read_dir(package_path)
        .map_err(PackageDiscoveryError::CannotReadDir)?
        .par_bridge()
        .try_fold(
            || (false, vec![], vec![]),
            |(mut is_package, mut files, mut dirs), entry| -> Result<_> {
                let entry = entry.map_err(PackageDiscoveryError::CannotReadDirEntry)?;
                let file_type = entry
                    .file_type()
                    .map_err(PackageDiscoveryError::CannotDetermineFileType)?;
                if file_type.is_file() {
                    let file_name = entry.file_name();
                    files.push(entry);
                    if file_name == "__init__.py" {
                        is_package = true;
                    }
                } else if file_type.is_dir() {
                    dirs.push(entry);
                }
                Ok((is_package, files, dirs))
            },
        )
        .try_reduce(
            || (false, vec![], vec![]),
            |(mut is_package, mut files, mut dirs), (chunk_is_package, chunk_files, chunk_dirs)| {
                is_package = is_package || chunk_is_package;
                files.extend(chunk_files);
                dirs.extend(chunk_dirs);
                return Ok((is_package, files, dirs));
            },
        )?;

    if !is_package {
        return Err(PackageDiscoveryError::NotAPythonPackage)?;
    }

    let pypath = get_pypath(root_package_path, package_path, false)?;
    let mut package = Package::new(pypath, package_path.to_owned());

    for module in files
        .par_iter()
        .filter(|file| file.path().extension().unwrap_or_default() == "py")
        .map(|file| {
            let pypath = get_pypath(root_package_path, &file.path(), true)?;
            Ok(Module::new(pypath, file.path().to_owned()))
        })
        .collect::<Result<Vec<_>>>()?
    {
        package.add_module(Arc::new(module));
    }

    for child in dirs
        .par_iter()
        .filter(|dir| !dir.file_name().to_str().unwrap().starts_with("."))
        .map(|dir| _discover_package(root_package_path, &dir.path()))
        .collect::<Vec<_>>()
    {
        match child {
            Ok(child) => {
                package.add_child(Arc::clone(&child));
            }
            Err(e) => match e.root_cause().downcast_ref::<PackageDiscoveryError>() {
                Some(PackageDiscoveryError::NotAPythonPackage) => continue,
                _ => return Err(e),
            },
        }
    }

    Ok(Arc::new(package))
}

fn get_pypath(root_package_path: &Path, path: &Path, is_file: bool) -> Result<String> {
    let root_path_part = root_package_path.file_name().unwrap().to_str().unwrap();

    let mut path_part = path
        .strip_prefix(root_package_path)
        .unwrap()
        .to_str()
        .unwrap();
    if is_file {
        path_part = path_part.strip_suffix(".py").unwrap();
    }

    if path_part.is_empty() {
        Ok(root_path_part.to_string())
    } else {
        Ok(format!("{root_path_part}.{path_part}").replace("/", "."))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_discover_package() {
        let root_package_path = Path::new("./example");

        let root_package = discover_package(root_package_path).unwrap();

        assert_eq!(root_package.pypath, "example");
        let root_modules = root_package
            .modules
            .iter()
            .map(|m| m.pypath.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(
            root_modules,
            [
                "example.__init__",
                "example.a",
                "example.b",
                "example.c",
                "example.d",
                "example.e",
                "example.z",
            ]
            .into_iter()
            .collect::<HashSet<_>>()
        );
        assert_eq!(root_package.children.len(), 5);

        let binding = root_package
            .children
            .iter()
            .filter(|child| child.pypath == "example.child")
            .collect::<Vec<_>>();
        let child_package_1 = binding.first().unwrap();

        assert_eq!(child_package_1.pypath, "example.child");
        let root_modules = child_package_1
            .modules
            .iter()
            .map(|m| m.pypath.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(
            root_modules,
            [
                "example.child.__init__",
                "example.child.c_a",
                "example.child.c_b",
                "example.child.c_c",
                "example.child.c_d",
                "example.child.c_e",
                "example.child.c_z",
            ]
            .into_iter()
            .collect::<HashSet<_>>()
        );
        assert_eq!(child_package_1.children.len(), 0);

        let binding = root_package
            .children
            .iter()
            .filter(|child| child.pypath == "example.child2")
            .collect::<Vec<_>>();
        let child_package_2 = binding.first().unwrap();

        assert_eq!(child_package_2.pypath, "example.child2");
        let root_modules = child_package_2
            .modules
            .iter()
            .map(|m| m.pypath.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(
            root_modules,
            ["example.child2.__init__",]
                .into_iter()
                .collect::<HashSet<_>>()
        );
        assert_eq!(child_package_2.children.len(), 0);
    }
}
