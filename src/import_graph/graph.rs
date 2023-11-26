use anyhow::Result;
use pathfinding::prelude::{bfs, bfs_reach};
use std::collections::HashSet;
use std::sync::Arc;

use super::errors::Error;
use super::import_discovery;
use super::indexing;
use super::package_discovery::{Module, Package};

#[derive(Debug)]
pub struct ImportGraph {
    pub(super) packages_by_pypath: indexing::PackagesByPypath,
    pub(super) modules_by_pypath: indexing::ModulesByPypath,
    pub(super) packages_by_module: indexing::PackagesByModule,
    pub(super) imports: import_discovery::Imports,
    pub(super) reverse_imports: import_discovery::Imports,
}

impl ImportGraph {
    pub fn packages(&self) -> HashSet<String> {
        self.packages_by_pypath
            .values()
            .map(|package| package.pypath.clone())
            .collect()
    }

    pub fn modules(&self) -> HashSet<String> {
        self.modules_by_pypath
            .values()
            .map(|module| module.pypath.clone())
            .collect()
    }

    pub fn package_from_module(&self, module: &str) -> Result<String> {
        let module = match self.modules_by_pypath.get(module) {
            Some(module) => module,
            None => Err(Error::ModuleNotFound(module.to_string()))?,
        };
        Ok(self
            .packages_by_module
            .get(module)
            .map(|p| p.pypath.clone())
            .unwrap())
    }

    pub fn child_packages(&self, package: &str) -> Result<HashSet<String>> {
        let package: &Package = match self.packages_by_pypath.get(package) {
            Some(package) => package,
            None => {
                return Err(Error::PackageNotFound(package.to_string()))?;
            }
        };
        let mut packages = HashSet::new();
        for child in package.children.iter() {
            packages.insert(child.pypath.clone());
        }
        Ok(packages)
    }

    pub fn child_modules(&self, package: &str) -> Result<HashSet<String>> {
        let package: &Package = match self.packages_by_pypath.get(package) {
            Some(package) => package,
            None => {
                return Err(Error::PackageNotFound(package.to_string()))?;
            }
        };
        let mut modules = HashSet::new();
        for module in package.modules.iter() {
            modules.insert(module.pypath.clone());
        }
        Ok(modules)
    }

    pub fn descendant_modules(&self, package: &str) -> Result<HashSet<String>> {
        Ok(self
            ._descendant_modules(package)?
            .iter()
            .map(|m| m.pypath.clone())
            .collect())
    }

    fn _descendant_modules(&self, package: &str) -> Result<HashSet<Arc<Module>>> {
        let package: &Package = match self.packages_by_pypath.get(package) {
            Some(package) => package,
            None => {
                return Err(Error::PackageNotFound(package.to_string()))?;
            }
        };
        let mut modules = HashSet::new();
        let mut q = vec![package];
        while let Some(package) = q.pop() {
            for module in package.modules.iter() {
                modules.insert(Arc::clone(module));
            }
            for child in package.children.iter() {
                q.push(child);
            }
        }
        Ok(modules)
    }

    pub fn modules_directly_imported_by(&self, module_or_package: &str) -> Result<HashSet<String>> {
        let from_modules = match self.packages_by_pypath.get(module_or_package) {
            Some(_) => self._descendant_modules(module_or_package)?,
            None => match self.modules_by_pypath.get(module_or_package) {
                Some(module) => HashSet::from([Arc::clone(module)]),
                None => Err(Error::ModuleNotFound(module_or_package.to_string()))?,
            },
        };
        let mut modules = HashSet::new();
        for from_module in from_modules {
            modules.extend(
                self.imports
                    .get(&from_module)
                    .unwrap()
                    .iter()
                    .map(|module| module.pypath.clone())
                    .collect::<Vec<_>>(),
            )
        }
        Ok(modules)
    }

    pub fn modules_that_directly_import(&self, module_or_package: &str) -> Result<HashSet<String>> {
        let to_modules = match self.packages_by_pypath.get(module_or_package) {
            Some(_) => self._descendant_modules(module_or_package)?,
            None => match self.modules_by_pypath.get(module_or_package) {
                Some(module) => HashSet::from([Arc::clone(module)]),
                None => Err(Error::ModuleNotFound(module_or_package.to_string()))?,
            },
        };
        let mut modules = HashSet::new();
        for to_module in to_modules {
            modules.extend(
                self.reverse_imports
                    .get(&to_module)
                    .unwrap()
                    .iter()
                    .map(|module| module.pypath.clone())
                    .collect::<Vec<_>>(),
            )
        }
        Ok(modules)
    }

    pub fn downstream_modules(&self, module_or_package: &str) -> Result<HashSet<String>> {
        let from_modules = match self.packages_by_pypath.get(module_or_package) {
            Some(_) => self._descendant_modules(module_or_package)?,
            None => match self.modules_by_pypath.get(module_or_package) {
                Some(module) => HashSet::from([Arc::clone(module)]),
                None => Err(Error::ModuleNotFound(module_or_package.to_string()))?,
            },
        };
        let mut downstream_modules = HashSet::new();
        for from_module in from_modules {
            let reachable_modules = bfs_reach(from_module, |module| {
                self.imports
                    .get(module)
                    .unwrap()
                    .iter()
                    .map(Arc::clone)
                    .collect::<Vec<_>>()
            })
            .map(|m| m.pypath.clone())
            .skip(1) // Remove starting module from the results.
            .collect::<Vec<_>>();
            downstream_modules.extend(reachable_modules);
        }
        Ok(downstream_modules)
    }

    pub fn upstream_modules(&self, module_or_package: &str) -> Result<HashSet<String>> {
        let to_modules = match self.packages_by_pypath.get(module_or_package) {
            Some(_) => self._descendant_modules(module_or_package)?,
            None => match self.modules_by_pypath.get(module_or_package) {
                Some(module) => HashSet::from([Arc::clone(module)]),
                None => Err(Error::ModuleNotFound(module_or_package.to_string()))?,
            },
        };
        let mut upstream_modules = HashSet::new();
        for to_module in to_modules {
            let reachable_modules = bfs_reach(to_module, |module| {
                self.reverse_imports
                    .get(module)
                    .unwrap()
                    .iter()
                    .map(Arc::clone)
                    .collect::<Vec<_>>()
            })
            .map(|m| m.pypath.clone())
            .skip(1) // Remove starting module from the results.
            .collect::<Vec<_>>();
            upstream_modules.extend(reachable_modules);
        }
        Ok(upstream_modules)
    }

    pub fn path_exists(
        &self,
        from_module_or_package: &str,
        to_module_or_package: &str,
    ) -> Result<bool> {
        let from_modules = match self.packages_by_pypath.get(from_module_or_package) {
            Some(_) => self._descendant_modules(from_module_or_package)?,
            None => match self.modules_by_pypath.get(from_module_or_package) {
                Some(module) => HashSet::from([Arc::clone(module)]),
                None => Err(Error::ModuleNotFound(from_module_or_package.to_string()))?,
            },
        };
        let to_modules = match self.packages_by_pypath.get(to_module_or_package) {
            Some(_) => self._descendant_modules(to_module_or_package)?,
            None => match self.modules_by_pypath.get(to_module_or_package) {
                Some(module) => HashSet::from([Arc::clone(module)]),
                None => Err(Error::ModuleNotFound(to_module_or_package.to_string()))?,
            },
        };
        for from_module in from_modules.iter() {
            for to_module in to_modules.iter() {
                if self
                    ._shortest_path(Arc::clone(from_module), Arc::clone(to_module))
                    .is_some()
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub fn shortest_path(&self, from_module: &str, to_module: &str) -> Result<Option<Vec<String>>> {
        let from_module = match self.modules_by_pypath.get(from_module) {
            Some(module) => Arc::clone(module),
            None => Err(Error::ModuleNotFound(from_module.to_string()))?,
        };
        let to_module = match self.modules_by_pypath.get(to_module) {
            Some(module) => Arc::clone(module),
            None => Err(Error::ModuleNotFound(to_module.to_string()))?,
        };
        Ok(self._shortest_path(from_module, to_module))
    }

    fn _shortest_path(
        &self,
        from_module: Arc<Module>,
        to_module: Arc<Module>,
    ) -> Option<Vec<String>> {
        let shortest_path = bfs(
            &from_module,
            |module| {
                self.imports
                    .get(module)
                    .unwrap()
                    .iter()
                    .map(Arc::clone)
                    .collect::<Vec<_>>()
            },
            |module| *module == to_module,
        );
        shortest_path.map(|shortest_path| shortest_path.iter().map(|m| m.pypath.clone()).collect())
    }
}
