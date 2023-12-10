use anyhow::Result;
use itertools::Itertools;
use pathfinding::prelude::{bfs, bfs_loop, bfs_reach};
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
            .map(|package| package.pypath.to_string())
            .collect()
    }

    pub fn modules(&self) -> HashSet<String> {
        self.modules_by_pypath
            .values()
            .map(|module| module.pypath.to_string())
            .collect()
    }

    pub fn package_from_module(&self, module: &str) -> Result<String> {
        let module = match self.modules_by_pypath.get(&module.to_string()) {
            Some(module) => module,
            None => Err(Error::ModuleNotFound(module.to_string()))?,
        };
        Ok(self
            .packages_by_module
            .get(module)
            .map(|p| p.pypath.to_string())
            .unwrap())
    }

    pub fn child_packages(&self, package: &str) -> Result<HashSet<String>> {
        let package: &Package = match self.packages_by_pypath.get(&package.to_string()) {
            Some(package) => package,
            None => {
                return Err(Error::PackageNotFound(package.to_string()))?;
            }
        };
        let mut packages = HashSet::new();
        for child in package.children.iter() {
            packages.insert(child.pypath.to_string());
        }
        Ok(packages)
    }

    pub fn child_modules(&self, package: &str) -> Result<HashSet<String>> {
        let package: &Package = match self.packages_by_pypath.get(&package.to_string()) {
            Some(package) => package,
            None => {
                return Err(Error::PackageNotFound(package.to_string()))?;
            }
        };
        let mut modules = HashSet::new();
        for module in package.modules.iter() {
            modules.insert(module.pypath.to_string());
        }
        Ok(modules)
    }

    pub fn descendant_packages(&self, package: &str) -> Result<HashSet<String>> {
        let package = match self.packages_by_pypath.get(&package.to_string()) {
            Some(package) => package,
            None => {
                return Err(Error::PackageNotFound(package.to_string()))?;
            }
        };
        Ok(self
            ._descendant_packages(package)?
            .iter()
            .map(|p| p.pypath.to_string())
            .collect())
    }

    fn _descendant_packages(&self, package: &Arc<Package>) -> Result<Vec<Arc<Package>>> {
        let mut packages = vec![];
        let mut q = vec![package];
        while let Some(package) = q.pop() {
            for child in package.children.iter() {
                packages.push(Arc::clone(child));
                q.push(child);
            }
        }
        Ok(packages)
    }

    pub fn descendant_modules(&self, package: &str) -> Result<HashSet<String>> {
        let package = match self.packages_by_pypath.get(&package.to_string()) {
            Some(package) => package,
            None => {
                return Err(Error::PackageNotFound(package.to_string()))?;
            }
        };
        Ok(self
            ._descendant_modules(package)?
            .iter()
            .map(|m| m.pypath.to_string())
            .collect())
    }

    fn _descendant_modules(&self, package: &Arc<Package>) -> Result<HashSet<Arc<Module>>> {
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
                    module.pypath.to_string(),
                    imported_modules
                        .iter()
                        .map(|imported_module| imported_module.pypath.to_string())
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

    pub fn direct_import_exists(
        &self,
        from_module_or_package: &str,
        to_module_or_package: &str,
    ) -> Result<bool> {
        let from_modules = match self
            .packages_by_pypath
            .get(&from_module_or_package.to_string())
        {
            Some(package) => self._descendant_modules(package)?,
            None => match self
                .modules_by_pypath
                .get(&from_module_or_package.to_string())
            {
                Some(module) => HashSet::from([Arc::clone(module)]),
                None => Err(Error::ModuleNotFound(from_module_or_package.to_string()))?,
            },
        };
        let to_modules = match self
            .packages_by_pypath
            .get(&to_module_or_package.to_string())
        {
            Some(package) => self._descendant_modules(package)?,
            None => match self
                .modules_by_pypath
                .get(&to_module_or_package.to_string())
            {
                Some(module) => HashSet::from([Arc::clone(module)]),
                None => Err(Error::ModuleNotFound(to_module_or_package.to_string()))?,
            },
        };
        for from_module in from_modules.iter() {
            for to_module in to_modules.iter() {
                if self._direct_import_exists(Arc::clone(from_module), Arc::clone(to_module)) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn _direct_import_exists(&self, from_module: Arc<Module>, to_module: Arc<Module>) -> bool {
        self.imports.get(&from_module).unwrap().contains(&to_module)
    }

    pub fn modules_directly_imported_by(&self, module_or_package: &str) -> Result<HashSet<String>> {
        let from_modules = match self.packages_by_pypath.get(&module_or_package.to_string()) {
            Some(package) => self._descendant_modules(package)?,
            None => match self.modules_by_pypath.get(&module_or_package.to_string()) {
                Some(module) => HashSet::from([Arc::clone(module)]),
                None => Err(Error::ModuleNotFound(module_or_package.to_string()))?,
            },
        };
        let mut modules = HashSet::new();
        for from_module in from_modules {
            modules.extend(self._modules_directly_imported_by(&from_module)?)
        }
        Ok(modules.iter().map(|m| m.pypath.to_string()).collect())
    }

    fn _modules_directly_imported_by(&self, module: &Arc<Module>) -> Result<HashSet<Arc<Module>>> {
        Ok(self.imports.get(module).unwrap().clone())
    }

    pub fn modules_that_directly_import(&self, module_or_package: &str) -> Result<HashSet<String>> {
        let to_modules = match self.packages_by_pypath.get(&module_or_package.to_string()) {
            Some(package) => self._descendant_modules(package)?,
            None => match self.modules_by_pypath.get(&module_or_package.to_string()) {
                Some(module) => HashSet::from([Arc::clone(module)]),
                None => Err(Error::ModuleNotFound(module_or_package.to_string()))?,
            },
        };
        let mut modules = HashSet::new();
        for to_module in to_modules {
            modules.extend(self._modules_that_directly_import(&to_module)?)
        }
        Ok(modules.iter().map(|m| m.pypath.to_string()).collect())
    }

    fn _modules_that_directly_import(&self, module: &Arc<Module>) -> Result<HashSet<Arc<Module>>> {
        Ok(self.reverse_imports.get(module).unwrap().clone())
    }

    pub fn downstream_modules(&self, module_or_package: &str) -> Result<HashSet<String>> {
        let from_modules = match self.packages_by_pypath.get(&module_or_package.to_string()) {
            Some(package) => self._descendant_modules(package)?,
            None => match self.modules_by_pypath.get(&module_or_package.to_string()) {
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
            .map(|m| m.pypath.to_string())
            .skip(1) // Remove starting module from the results.
            .collect::<Vec<_>>();
            downstream_modules.extend(reachable_modules);
        }
        Ok(downstream_modules)
    }

    pub fn upstream_modules(&self, module_or_package: &str) -> Result<HashSet<String>> {
        let to_modules = match self.packages_by_pypath.get(&module_or_package.to_string()) {
            Some(package) => self._descendant_modules(package)?,
            None => match self.modules_by_pypath.get(&module_or_package.to_string()) {
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
            .map(|m| m.pypath.to_string())
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
        let from_modules = match self
            .packages_by_pypath
            .get(&from_module_or_package.to_string())
        {
            Some(package) => self._descendant_modules(package)?,
            None => match self
                .modules_by_pypath
                .get(&from_module_or_package.to_string())
            {
                Some(module) => HashSet::from([Arc::clone(module)]),
                None => Err(Error::ModuleNotFound(from_module_or_package.to_string()))?,
            },
        };
        let to_modules = match self
            .packages_by_pypath
            .get(&to_module_or_package.to_string())
        {
            Some(package) => self._descendant_modules(package)?,
            None => match self
                .modules_by_pypath
                .get(&to_module_or_package.to_string())
            {
                Some(module) => HashSet::from([Arc::clone(module)]),
                None => Err(Error::ModuleNotFound(to_module_or_package.to_string()))?,
            },
        };
        for from_module in from_modules.iter() {
            for to_module in to_modules.iter() {
                if self._path_exists(Arc::clone(from_module), Arc::clone(to_module)) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn _path_exists(&self, from_module: Arc<Module>, to_module: Arc<Module>) -> bool {
        self._shortest_path(from_module, to_module).is_some()
    }

    pub fn shortest_path(&self, from_module: &str, to_module: &str) -> Result<Option<Vec<String>>> {
        let from_module = match self.modules_by_pypath.get(&from_module.to_string()) {
            Some(module) => Arc::clone(module),
            None => Err(Error::ModuleNotFound(from_module.to_string()))?,
        };
        let to_module = match self.modules_by_pypath.get(&to_module.to_string()) {
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
        if from_module == to_module {
            let shortest_path = bfs_loop(&from_module, |module| {
                self.imports
                    .get(module)
                    .unwrap()
                    .iter()
                    .map(Arc::clone)
                    .collect::<Vec<_>>()
            });
            shortest_path
                .map(|shortest_path| shortest_path.iter().map(|m| m.pypath.to_string()).collect())
        } else {
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
            shortest_path
                .map(|shortest_path| shortest_path.iter().map(|m| m.pypath.to_string()).collect())
        }
    }

    pub fn ignore_imports<'a>(
        &self,
        imports_to_remove: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<ImportGraph> {
        let mut imports_to_remove_ = vec![];
        for (from_module, to_module) in imports_to_remove {
            let from_module = match self.modules_by_pypath.get(&from_module.to_string()) {
                Some(module) => Arc::clone(module),
                None => Err(Error::ModuleNotFound(from_module.to_string()))?,
            };
            let to_module = match self.modules_by_pypath.get(&to_module.to_string()) {
                Some(module) => Arc::clone(module),
                None => Err(Error::ModuleNotFound(to_module.to_string()))?,
            };
            if self.imports.get(&from_module).unwrap().contains(&to_module) {
                imports_to_remove_.push((from_module, to_module));
            } else {
                return Err(Error::ImportNotFound(
                    from_module.pypath.to_string(),
                    to_module.pypath.to_string(),
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
        import_graph.reverse_imports = indexing::reverse_imports(&import_graph.imports)?;
        Ok(import_graph)
    }

    pub fn subgraph(&self, package: &str) -> Result<ImportGraph> {
        let package = match self.packages_by_pypath.get(&package.to_string()) {
            Some(package) => package,
            None => {
                return Err(Error::PackageNotFound(package.to_string()))?;
            }
        };
        let packages = {
            let mut packages = self._descendant_packages(package)?;
            packages.push(Arc::clone(package));
            packages
        };
        let modules = self._descendant_modules(package)?;

        let packages_by_pypath = packages
            .iter()
            .map(|p| (Arc::clone(&p.pypath), Arc::clone(p)))
            .collect::<HashMap<_, _>>();

        let modules_by_pypath = modules
            .iter()
            .map(|m| (Arc::clone(&m.pypath), Arc::clone(m)))
            .collect::<HashMap<_, _>>();

        let packages_by_module = self
            .packages_by_module
            .iter()
            .filter(|(m, _)| modules.contains(&Arc::clone(m)))
            .map(|(m, p)| (Arc::clone(m), Arc::clone(p)))
            .collect();

        let imports = modules.iter().cartesian_product(modules.iter()).fold(
            HashMap::new(),
            |mut hm, (m1, m2)| {
                let entry: &mut HashSet<Arc<Module>> = hm.entry(Arc::clone(m1)).or_default();
                if self._direct_import_exists(Arc::clone(m1), Arc::clone(m2)) {
                    entry.insert(Arc::clone(m2));
                }
                hm
            },
        );

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
        let package = match self.packages_by_pypath.get(&package.to_string()) {
            Some(package) => package,
            None => {
                return Err(Error::PackageNotFound(package.to_string()))?;
            }
        };
        let init_module = self._init_module(package);

        let packages_to_remove = self._descendant_packages(package)?;
        let modules_to_remove = {
            let mut descendant_modules = self._descendant_modules(package)?;
            descendant_modules.remove(&init_module);
            descendant_modules
        };

        let mut import_graph = self.clone();

        for module_to_remove in modules_to_remove.iter() {
            for imported_module in import_graph._modules_directly_imported_by(module_to_remove)? {
                import_graph
                    .imports
                    .get_mut(&init_module)
                    .unwrap()
                    .insert(Arc::clone(&imported_module));
                import_graph
                    .reverse_imports
                    .get_mut(&imported_module)
                    .unwrap()
                    .insert(Arc::clone(&init_module));
            }
            for importing_module in import_graph._modules_that_directly_import(module_to_remove)? {
                import_graph
                    .imports
                    .get_mut(&importing_module)
                    .unwrap()
                    .insert(Arc::clone(&init_module));
                import_graph
                    .reverse_imports
                    .get_mut(&init_module)
                    .unwrap()
                    .insert(Arc::clone(&importing_module));
            }
        }

        for module_to_remove in modules_to_remove.iter() {
            import_graph.imports.remove(module_to_remove);
            for imported_modules in import_graph.imports.values_mut() {
                imported_modules.remove(module_to_remove);
            }
            import_graph.reverse_imports.remove(module_to_remove);
            for importing_modules in import_graph.reverse_imports.values_mut() {
                importing_modules.remove(module_to_remove);
            }
            import_graph
                .modules_by_pypath
                .remove(&module_to_remove.pypath);
            import_graph.packages_by_module.remove(module_to_remove);
        }
        for package_to_remove in packages_to_remove {
            import_graph
                .packages_by_pypath
                .remove(&package_to_remove.pypath);
        }

        // Remove self imports.
        for (module, imported_modules) in import_graph.imports.iter_mut() {
            imported_modules.remove(module);
        }
        for (module, importing_modules) in import_graph.reverse_imports.iter_mut() {
            importing_modules.remove(module);
        }

        Ok(import_graph)
    }

    fn _init_module(&self, package: &Arc<Package>) -> Arc<Module> {
        package
            .modules
            .iter()
            .filter(|m| m.pypath.ends_with(".__init__"))
            .map(Arc::clone)
            .next()
            .unwrap()
    }
}
