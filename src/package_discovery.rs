use std::{collections::{HashMap, HashSet}, path::{Path, PathBuf}};
use anyhow::Result;
use std::fs;

pub fn discover_package(path: &Path) -> Result<PackageInfo> {
    let mut packages = vec![Package::new(path.strip_prefix(path)?)?];
    let mut modules = vec![];
    let mut package_to_modules = HashMap::from([(0, HashSet::new())]);
    let mut module_to_package = HashMap::new();

    let mut packages_to_discover = vec![0];
    while let Some(parent_package_idx) = packages_to_discover.pop() {
        let parent_package = packages.get(parent_package_idx).unwrap();

        for entry in fs::read_dir(path.join(&parent_package.path))? {
            let entry = entry?;
            if entry.path().is_dir() {
                packages.push(Package::new(entry.path().strip_prefix(path)?)?);
                package_to_modules.insert(packages.len()-1, HashSet::new());
                packages_to_discover.push(packages.len()-1);
            } else if entry.path().is_file() {
                modules.push(Module::new(entry.path().strip_prefix(path)?)?);
                package_to_modules.get_mut(&parent_package_idx).unwrap().insert(modules.len()-1);
                module_to_package.insert(modules.len()-1, parent_package_idx);
            }
        }
    }

    let package_info = PackageInfo{
        packages,
        modules,
        package_to_modules,
        module_to_package,
    };
    println!("{:#?}", package_info);

    Ok(package_info)
}

fn path_to_pypath(path: &Path) -> Result<String> {
    let mut s = path.to_str().unwrap().to_owned();
    if s.ends_with(".py") {
        s =  s.strip_suffix(".py").unwrap().to_owned();
    }
    s = s.replace("/", ".");
    Ok(s.to_owned())
}

#[derive(Debug)]
pub struct PackageInfo {
    packages: Vec<Package>,
    modules: Vec<Module>,
    package_to_modules: HashMap<usize, HashSet<usize>>,
    module_to_package: HashMap<usize, usize>,
}

#[derive(Debug)]
struct Package {
    pypath: String,
    path: PathBuf,
}

impl Package {
    fn new(path: &Path) -> Result<Package> {
        Ok(Package {
            pypath: path_to_pypath(path)?,
            path: path.to_owned(),
        })
    }
}

#[derive(Debug)]
struct Module {
    path: PathBuf,
    pypath: String,
}

impl Module {
    fn new(path: &Path) -> Result<Module> {
        Ok(Module {
            path: path.to_owned(),
            pypath: path_to_pypath(path)?,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;
    use crate::testutils::TestPackage;

    #[test]
    fn test_discover_package() -> Result<()> {
        let test_package = TestPackage::new(hashmap! {
            "__init__" => "",
            "main" => "",
            "colors.__init__" => "",
            "colors.red" => "",
            "food.__init__" => "",
            "food.pizza" => "",
            "food.fruit.__init__" => "",
            "food.fruit.apple" => "",
        })?;

        discover_package(test_package.path());
        
        Ok(())
    }
}