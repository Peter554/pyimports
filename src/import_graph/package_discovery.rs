use anyhow::Result;
use rayon::prelude::*;
use std::path::PathBuf;
use std::{fs, path::Path};

use super::errors::Error;

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Module {
    pub pypath: String,
    pub(super) path: PathBuf,
}

impl Module {
    fn new(pypath: String, path: PathBuf) -> Self {
        Module { pypath, path }
    }
}

pub fn discover_package(root_package_path: PathBuf) -> Result<Package> {
    _discover_package(root_package_path.as_path(), root_package_path.as_path())
}

fn _discover_package(root_package_path: &Path, package_path: &Path) -> Result<Package> {
    let (is_package, files, dirs) = fs::read_dir(package_path)
        .map_err(Error::CannotReadDir)?
        .par_bridge()
        .try_fold(
            || (false, vec![], vec![]),
            |(mut is_package, mut files, mut dirs), entry| -> Result<_> {
                let entry = entry?;
                let file_type = entry.file_type()?;
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
                Ok((is_package, files, dirs))
            },
        )?;

    if !is_package {
        Err(Error::NotAPythonPackage)?;
    }

    let pypath = get_pypath(root_package_path, package_path, false)?;
    let mut package = Package::new(pypath);

    for module in files
        .par_iter()
        .filter(|file| file.path().extension().unwrap_or_default() == "py")
        .map(|file| {
            let pypath = get_pypath(root_package_path, &file.path(), true)?;
            Ok(Module::new(pypath, file.path().to_path_buf()))
        })
        .collect::<Result<Vec<_>>>()?
    {
        package.add_module(module);
    }

    for child in dirs
        .par_iter()
        .filter(|dir| !dir.file_name().to_str().unwrap().starts_with('.'))
        .map(|dir| _discover_package(root_package_path, &dir.path()))
        .collect::<Vec<_>>()
    {
        match child {
            Ok(child) => {
                package.add_child(child);
            }
            Err(e) => match e.root_cause().downcast_ref::<Error>() {
                Some(Error::NotAPythonPackage) => continue,
                _ => return Err(e),
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
        Ok(format!("{root_path_part}.{path_part}").replace('/', "."))
    }
}
