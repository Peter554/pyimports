use anyhow::Result;
use pathfinding::prelude::{bfs, bfs_reach};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use super::errors::Error;
use super::import_discovery;
use super::indexing;
use super::package_discovery::{Module, Package};

#[derive(Debug, Clone)]
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

    pub fn descendant_packages(&self, package: &str) -> Result<HashSet<String>> {
        Ok(self
            ._descendant_packages(package)?
            .iter()
            .map(|p| p.pypath.clone())
            .collect())
    }

    fn _descendant_packages(&self, package: &str) -> Result<HashSet<Arc<Package>>> {
        let package: &Package = match self.packages_by_pypath.get(package) {
            Some(package) => package,
            None => {
                return Err(Error::PackageNotFound(package.to_string()))?;
            }
        };
        let mut packages = HashSet::new();
        let mut q = vec![package];
        while let Some(package) = q.pop() {
            for child in package.children.iter() {
                packages.insert(Arc::clone(child));
                q.push(child);
            }
        }
        Ok(packages)
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

    pub fn direct_imports(&self) -> HashMap<String, HashSet<String>> {
        self.imports
            .iter()
            .map(|(module, imported_modules)| {
                (
                    module.pypath.clone(),
                    imported_modules
                        .iter()
                        .map(|imported_module| imported_module.pypath.clone())
                        .collect(),
                )
            })
            .collect()
    }

    pub fn direct_imports_flat(&self) -> HashSet<(String, String)> {
        self.direct_imports()
            .into_iter()
            .flat_map(|(module, imported_modules)| {
                imported_modules
                    .into_iter()
                    .map(|imported_module| (module.clone(), imported_module))
                    .collect::<Vec<_>>()
            })
            .collect()
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

    pub fn ignore_imports<'a>(
        &self,
        imports_to_remove: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<ImportGraph> {
        let mut imports_to_remove_ = vec![];
        for (from_module, to_module) in imports_to_remove {
            let from_module = match self.modules_by_pypath.get(from_module) {
                Some(module) => Arc::clone(module),
                None => Err(Error::ModuleNotFound(from_module.to_string()))?,
            };
            let to_module = match self.modules_by_pypath.get(to_module) {
                Some(module) => Arc::clone(module),
                None => Err(Error::ModuleNotFound(to_module.to_string()))?,
            };
            if self.imports.get(&from_module).unwrap().contains(&to_module) {
                imports_to_remove_.push((from_module, to_module));
            } else {
                return Err(Error::ImportNotFound(
                    from_module.pypath.clone(),
                    to_module.pypath.clone(),
                ))?;
            }
        }
        let mut import_graph = self.clone();
        for (from_module, to_module) in imports_to_remove_ {
            import_graph
                .imports
                .get_mut(&from_module)
                .unwrap()
                .remove(&to_module);
        }
        Ok(import_graph)
    }

    pub fn subgraph(&self, package: &str) -> Result<ImportGraph> {
        let this_package = match self.packages_by_pypath.get(package) {
            Some(package) => package,
            None => {
                return Err(Error::PackageNotFound(package.to_string()))?;
            }
        };

        let packages_to_keep = {
            let mut packages_to_keep = self._descendant_packages(package)?;
            packages_to_keep.insert(Arc::clone(this_package));
            packages_to_keep
        };
        let package_pypaths_to_keep = packages_to_keep
            .iter()
            .map(|p| p.pypath.clone())
            .collect::<HashSet<_>>();
        let modules_to_keep = self._descendant_modules(package)?;
        let module_pypaths_to_keep = modules_to_keep
            .iter()
            .map(|m| m.pypath.clone())
            .collect::<HashSet<_>>();

        let mut packages_by_pypath = self.packages_by_pypath.clone();
        for pypath in packages_by_pypath.clone().keys() {
            if !package_pypaths_to_keep.contains(pypath) {
                packages_by_pypath.remove(pypath);
            }
        }

        let mut modules_by_pypath = self.modules_by_pypath.clone();
        for pypath in modules_by_pypath.clone().keys() {
            if !module_pypaths_to_keep.contains(pypath) {
                modules_by_pypath.remove(pypath);
            }
        }

        let mut packages_by_module = self.packages_by_module.clone();
        for (module, package) in packages_by_module.clone().iter() {
            if !modules_to_keep.contains(module) || !packages_to_keep.contains(package) {
                packages_by_module.remove(module);
            }
        }

        let mut imports = self.imports.clone();
        for (module, imported_modules) in imports.clone().iter() {
            if !modules_to_keep.contains(module) {
                imports.remove(module);
                continue;
            }
            for imported_module in imported_modules.clone().iter() {
                if !modules_to_keep.contains(imported_module) {
                    imports.get_mut(module).unwrap().remove(imported_module);
                }
            }
        }

        let reverse_imports = indexing::reverse_imports(&imports)?;

        Ok(ImportGraph {
            packages_by_pypath,
            modules_by_pypath,
            packages_by_module,
            imports,
            reverse_imports,
        })
    }

    pub fn squash_package(&self, package: &str) -> Result<ImportGraph> {
        let package_to_squash = match self.packages_by_pypath.get(package) {
            Some(package) => package,
            None => {
                return Err(Error::PackageNotFound(package.to_string()))?;
            }
        };
        let binding = self
            .child_modules(package)?
            .into_iter()
            .filter(|m| m.ends_with(".__init__"))
            .collect::<Vec<_>>();
        let init_module_pypath = binding.first().unwrap();
        let init_module = self.modules_by_pypath.get(init_module_pypath).unwrap();

        let packages_to_replace = self._descendant_packages(package)?;
        let package_pypaths_to_replace = packages_to_replace
            .iter()
            .map(|p| p.pypath.clone())
            .collect::<HashSet<_>>();
        let modules_to_replace = {
            let mut modules_to_replace = self._descendant_modules(package)?;
            modules_to_replace.remove(init_module);
            modules_to_replace
        };
        let module_pypaths_to_replace = modules_to_replace
            .iter()
            .map(|m| m.pypath.clone())
            .collect::<HashSet<_>>();

        let mut packages_by_pypath = self.packages_by_pypath.clone();
        for pypath in packages_by_pypath.clone().keys() {
            if package_pypaths_to_replace.contains(pypath) {
                packages_by_pypath.remove(pypath);
            }
        }

        let mut modules_by_pypath = self.modules_by_pypath.clone();
        for pypath in modules_by_pypath.clone().keys() {
            if module_pypaths_to_replace.contains(pypath) {
                modules_by_pypath.remove(pypath);
            }
        }

        let mut packages_by_module = self.packages_by_module.clone();
        for (module, package) in packages_by_module.clone().iter() {
            if modules_to_replace.contains(module) && packages_to_replace.contains(package) {
                packages_by_module.remove(module);
                packages_by_module.insert(Arc::clone(init_module), Arc::clone(package_to_squash));
            } else if modules_to_replace.contains(module) {
                packages_by_module.remove(module);
                packages_by_module.insert(Arc::clone(init_module), Arc::clone(package));
            } else if packages_to_replace.contains(package) {
                packages_by_module.remove(module);
                packages_by_module.insert(Arc::clone(module), Arc::clone(package_to_squash));
            }
        }

        let mut imports = self.imports.clone();
        for (module, imported_modules) in imports.clone().iter_mut() {
            for imported_module in imported_modules.clone().iter() {
                if modules_to_replace.contains(imported_module) {
                    imported_modules.remove(imported_module);
                    imported_modules.insert(Arc::clone(init_module));
                }
            }
            imports.insert(Arc::clone(module), imported_modules.clone());
            if modules_to_replace.contains(module) {
                imports.remove(module);
                imports
                    .get_mut(&Arc::clone(init_module))
                    .unwrap()
                    .extend(imported_modules.clone());
            }
        }

        let reverse_imports = indexing::reverse_imports(&imports)?;

        Ok(ImportGraph {
            packages_by_pypath,
            modules_by_pypath,
            packages_by_module,
            imports,
            reverse_imports,
        })
    }
}
