use anyhow::Result;
use rayon::prelude::*;
use std::{fs, path::Path, io};
use thiserror::Error;

#[derive(Error,Debug)]
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

#[derive(Debug)]
pub struct Package {
    pub pypath: String,
    pub children: Vec<Package>,
    pub modules: Vec<Module>,
}

impl Package {
    fn new(pypath: String) -> Self {
        Package {
            pypath,
            children: vec![],
            modules: vec![],
        }
    }

    fn add_child(&mut self, child: Package) {
        self.children.push(child);
    }

    fn add_module(&mut self, module: Module) {
        self.modules.push(module);
    }
}

#[derive(Debug)]
pub struct Module {
    pub pypath: String,
}

impl Module {
    fn new(pypath: String) -> Self {
        Module { pypath }
    }
}

pub fn discover_package(package_path: &Path) -> Result<Package> {
    _discover_package(package_path, package_path)
}

fn _discover_package(root_package_path: &Path, package_path: &Path) -> Result<Package> {
    let (is_package, files, dirs) = fs::read_dir(package_path)
        .map_err(PackageDiscoveryError::CannotReadDir)?
        .par_bridge()
        .try_fold(
            || (false, vec![], vec![]),
            |(mut is_package, mut files, mut dirs), entry| -> Result<_>{
                let entry = entry.map_err(PackageDiscoveryError::CannotReadDirEntry)?;
                let file_type = entry.file_type().map_err(PackageDiscoveryError::CannotDetermineFileType)?;
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
    let mut package = Package::new(pypath);

    for module in files
        .par_iter()
        .filter(|file| file.path().extension().unwrap_or_default() == "py")
        .map(|file| {
            let pypath = get_pypath(root_package_path, &file.path(), true)?;
            Ok(Module::new(pypath))
        })
        .collect::<Result<Vec<_>>>()?
    {
        package.add_module(module);
    }

    for child in dirs
        .par_iter()
        .filter(|dir| !dir.file_name().to_str().unwrap().starts_with("."))
        .map(|dir| _discover_package(root_package_path, &dir.path()))
        .collect::<Vec<_>>()
    {
        match child {
            Ok(child) => {
                package.add_child(child);
            }
            Err(e) => {
                match e.root_cause().downcast_ref::<PackageDiscoveryError>() {
                    Some(PackageDiscoveryError::NotAPythonPackage) => continue,
                    _ => return Err(e)
                }
            },
        }
    }

    Ok(package)
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
            ["example.__init__", "example.a", "example.b",]
                .into_iter()
                .collect::<HashSet<_>>()
        );
        assert_eq!(root_package.children.len(), 1);

        let child_package = root_package.children.first().unwrap();

        assert_eq!(child_package.pypath, "example.child");
        let root_modules = child_package
            .modules
            .iter()
            .map(|m| m.pypath.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(
            root_modules,
            [
                "example.child.__init__",
                "example.child.c",
                "example.child.d",
            ]
            .into_iter()
            .collect::<HashSet<_>>()
        );
        assert_eq!(child_package.children.len(), 0);
    }
}
