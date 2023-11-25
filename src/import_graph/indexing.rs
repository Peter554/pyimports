use anyhow::Result;
use std::collections::{HashMap, HashSet};

use super::{
    import_discovery::Imports,
    package_discovery::{Module, Package},
};

pub type PackagesByPypath<'a> = HashMap<&'a str, &'a Package>;
pub type ModulesByPypath<'a> = HashMap<&'a str, &'a Module>;
pub type PackagesByModule<'a> = HashMap<&'a Module, &'a Package>;

pub fn get_packages_by_pypath(root_package: &Package) -> Result<PackagesByPypath> {
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

pub fn get_modules_by_pypath(root_package: &Package) -> Result<ModulesByPypath> {
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

pub fn get_packages_by_module(root_package: &Package) -> Result<PackagesByModule> {
    let mut m = HashMap::new();
    let mut q = vec![root_package];
    while let Some(package) = q.pop() {
        for module in package.modules.iter() {
            m.insert(module, package);
        }
        for child in package.children.iter() {
            q.push(child);
        }
    }
    Ok(m)
}

pub fn reverse_imports<'a>(imports: &'a Imports<'a>) -> Result<Imports<'a>> {
    let mut hm = HashMap::new();
    for (module, imports) in imports.iter() {
        hm.entry(module as &Module).or_insert(HashSet::new());
        for import in imports.iter() {
            hm.entry(import as &Module)
                .or_insert(HashSet::new())
                .insert(module as &Module);
        }
    }
    Ok(hm)
}
