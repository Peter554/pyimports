use anyhow::Result;
use std::collections::HashMap;

use crate::package_discovery;

pub type PackagesByPypath<'a> = HashMap<&'a str, &'a package_discovery::Package>;
pub type ModulesByPypath<'a> = HashMap<&'a str, &'a package_discovery::Module>;

pub fn get_packages_by_pypath(
    root_package: &package_discovery::Package,
) -> Result<PackagesByPypath> {
    let mut m = HashMap::new();
    let mut q = vec![root_package];
    while let Some(package) = q.pop() {
        m.insert(package.pypath.as_str(), package);
        for child in package.children.iter() {
            q.push(child);
        }
    }
    Ok(m)
}

pub fn get_modules_by_pypath(root_package: &package_discovery::Package) -> Result<ModulesByPypath> {
    let mut m = HashMap::new();
    let mut q = vec![root_package];
    while let Some(package) = q.pop() {
        for module in package.modules.iter() {
            m.insert(module.pypath.as_str(), module);
        }
        for child in package.children.iter() {
            q.push(child);
        }
    }
    Ok(m)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::package_discovery;

    #[test]
    fn test_get_packages_by_pypath() {
        let root_package_path = Path::new("./example");
        let root_package = package_discovery::discover_package(root_package_path).unwrap();

        let packages_by_pypath = get_packages_by_pypath(&root_package).unwrap();

        assert_eq!(packages_by_pypath.len(), 6);
        assert_eq!(packages_by_pypath.get("example").unwrap(), &&root_package);
        assert_eq!(
            packages_by_pypath.get("example.child").unwrap(),
            root_package
                .children
                .iter()
                .filter(|child| child.pypath == "example.child")
                .collect::<Vec<_>>()
                .first()
                .unwrap()
        );
        assert_eq!(
            packages_by_pypath.get("example.child2").unwrap(),
            root_package
                .children
                .iter()
                .filter(|child| child.pypath == "example.child2")
                .collect::<Vec<_>>()
                .first()
                .unwrap()
        );
    }

    #[test]
    fn test_get_modules_by_pypath() {
        let root_package_path = Path::new("./example");
        let root_package = package_discovery::discover_package(root_package_path).unwrap();

        let modules_by_pypath = get_modules_by_pypath(&root_package).unwrap();

        assert_eq!(modules_by_pypath.len(), 18);
        for module in root_package.modules.iter() {
            assert_eq!(
                modules_by_pypath.get(module.pypath.as_str()).unwrap(),
                &module
            )
        }
        for child_module in root_package
            .children
            .iter()
            .filter(|child| child.pypath == "example.child")
            .collect::<Vec<_>>()
            .first()
            .unwrap()
            .modules
            .iter()
        {
            assert_eq!(
                modules_by_pypath.get(child_module.pypath.as_str()).unwrap(),
                &child_module
            )
        }
    }
}
