use anyhow::Result;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use super::{
    import_discovery::Imports,
    package_discovery::{Module, Package},
};

pub type PackagesByPypath = HashMap<Arc<String>, Arc<Package>>;
pub type ModulesByPypath = HashMap<Arc<String>, Arc<Module>>;
pub type PackagesByModule = HashMap<Arc<Module>, Arc<Package>>;

pub fn get_packages_by_pypath(root_package: Arc<Package>) -> Result<PackagesByPypath> {
    let mut m = HashMap::new();
    let mut q = vec![root_package];
    while let Some(package) = q.pop() {
        m.insert(package.pypath.clone(), Arc::clone(&package));
        for child in package.children.iter() {
            q.push(Arc::clone(child));
        }
    }
    Ok(m)
}

pub fn get_modules_by_pypath(root_package: Arc<Package>) -> Result<ModulesByPypath> {
    let mut m = HashMap::new();
    let mut q = vec![root_package];
    while let Some(package) = q.pop() {
        for module in package.modules.iter() {
            m.insert(module.pypath.clone(), Arc::clone(module));
        }
        for child in package.children.iter() {
            q.push(Arc::clone(child));
        }
    }
    Ok(m)
}

pub fn get_packages_by_module(root_package: Arc<Package>) -> Result<PackagesByModule> {
    let mut m = HashMap::new();
    let mut q = vec![root_package];
    while let Some(package) = q.pop() {
        for module in package.modules.iter() {
            m.insert(Arc::clone(module), Arc::clone(&package));
        }
        for child in package.children.iter() {
            q.push(Arc::clone(child));
        }
    }
    Ok(m)
}

pub fn reverse_imports(imports: &Imports) -> Result<Imports> {
    let mut hm = HashMap::new();
    for (module, imports) in imports.iter() {
        hm.entry(Arc::clone(module)).or_insert(HashSet::new());
        for import in imports.iter() {
            hm.entry(Arc::clone(import))
                .or_insert(HashSet::new())
                .insert(Arc::clone(module));
        }
    }
    Ok(hm)
}
