use anyhow::Result;
use std::{collections::HashMap, sync::Arc};

use crate::package_discovery;

pub type PackagesByPypath = HashMap<String, Arc<package_discovery::Package>>;
pub type ModulesByPypath = HashMap<String, Arc<package_discovery::Module>>;

pub fn get_packages_by_pypath(
    root_package: Arc<package_discovery::Package>,
) -> Result<PackagesByPypath> {
    let mut m = HashMap::new();
    let mut q = vec![root_package];
    while let Some(package) = q.pop() {
        m.insert(package.pypath.clone(), Arc::clone(&package));
        for child in package.children.iter() {
            q.push(Arc::clone(&child));
        }
    }
    Ok(m)
}

pub fn get_modules_by_pypath(
    root_package: Arc<package_discovery::Package>,
) -> Result<ModulesByPypath> {
    let mut m = HashMap::new();
    let mut q = vec![root_package];
    while let Some(package) = q.pop() {
        for module in package.modules.iter() {
            m.insert(module.pypath.clone(), Arc::clone(module));
        }
        for child in package.children.iter() {
            q.push(Arc::clone(&child));
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

        let mut packages_by_pypath = get_packages_by_pypath(Arc::clone(&root_package)).unwrap();

        assert_eq!(packages_by_pypath.len(), 6);
        assert_eq!(
            packages_by_pypath.remove("example").unwrap(),
            Arc::clone(&root_package)
        );
        assert_eq!(
            packages_by_pypath.remove("example.child").unwrap(),
            Arc::clone(
                &root_package
                    .children
                    .iter()
                    .filter(|child| child.pypath == "example.child")
                    .collect::<Vec<_>>()
                    .first()
                    .unwrap()
            )
        );
        assert_eq!(
            packages_by_pypath.remove("example.child2").unwrap(),
            Arc::clone(
                &root_package
                    .children
                    .iter()
                    .filter(|child| child.pypath == "example.child2")
                    .collect::<Vec<_>>()
                    .first()
                    .unwrap()
            )
        );
    }

    #[test]
    fn test_get_modules_by_pypath() {
        let root_package_path = Path::new("./example");
        let root_package = package_discovery::discover_package(root_package_path).unwrap();

        let mut modules_by_pypath = get_modules_by_pypath(Arc::clone(&root_package)).unwrap();

        assert_eq!(modules_by_pypath.len(), 18);
        for module in root_package.modules.iter() {
            assert_eq!(
                modules_by_pypath.remove(&module.pypath).unwrap(),
                Arc::clone(module)
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
                modules_by_pypath.remove(&child_module.pypath).unwrap(),
                Arc::clone(child_module)
            )
        }
    }
}
