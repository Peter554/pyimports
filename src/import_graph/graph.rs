use anyhow::Result;
use ouroboros::self_referencing;
use pathfinding::prelude::{bfs, bfs_reach};
use std::collections::HashSet;

use super::builder;
use super::errors::Error;
use super::import_discovery;
use super::indexing;
use super::package_discovery::{self, Module, Package};

#[self_referencing]
#[derive(Debug)]
pub struct ImportGraph {
    root_package: package_discovery::Package,

    #[borrows(root_package)]
    #[covariant]
    packages_by_pypath: indexing::PackagesByPypath<'this>,

    #[borrows(root_package)]
    #[covariant]
    modules_by_pypath: indexing::ModulesByPypath<'this>,

    #[borrows(root_package)]
    #[covariant]
    packages_by_module: indexing::PackagesByModule<'this>,

    #[borrows(root_package, modules_by_pypath)]
    #[covariant]
    imports: import_discovery::Imports<'this>,

    #[borrows(imports)]
    #[covariant]
    reverse_imports: import_discovery::Imports<'this>,
}

impl ImportGraph {
    pub(super) fn build(builder: builder::ImportGraphBuilder) -> Result<Self> {
        let import_graph = ImportGraphTryBuilder {
            root_package: package_discovery::discover_package(builder.root_package_path.clone())?,
            packages_by_pypath_builder: |root_package| {
                indexing::get_packages_by_pypath(root_package)
            },
            modules_by_pypath_builder: indexing::get_modules_by_pypath,
            packages_by_module_builder: |root_package| {
                indexing::get_packages_by_module(root_package)
            },
            imports_builder: |root_package, modules_by_pypath| {
                import_discovery::discover_imports(root_package, modules_by_pypath)
            },
            reverse_imports_builder: indexing::reverse_imports,
        }
        .try_build()?;
        Ok(import_graph)
    }

    pub fn packages(&self) -> HashSet<&str> {
        self.borrow_packages_by_pypath()
            .values()
            .map(|package| package.pypath.as_str())
            .collect()
    }

    pub fn modules(&self) -> HashSet<&str> {
        self.borrow_modules_by_pypath()
            .values()
            .map(|module| module.pypath.as_str())
            .collect()
    }

    pub fn package_from_module(&self, module: &str) -> Result<&str> {
        let module = match self.borrow_modules_by_pypath().get(module) {
            Some(module) => module as &Module,
            None => Err(Error::ModuleNotFound(module.to_string()))?,
        };
        Ok(self
            .borrow_packages_by_module()
            .get(module)
            .map(|p| p.pypath.as_str())
            .unwrap())
    }

    pub fn packages_from_modules(&self, modules: HashSet<&str>) -> Result<HashSet<&str>> {
        let mut packages = HashSet::new();
        for module in modules.iter() {
            let module = match self.borrow_modules_by_pypath().get(module) {
                Some(module) => module as &Module,
                None => Err(Error::ModuleNotFound(module.to_string()))?,
            };
            packages.insert(
                self.borrow_packages_by_module()
                    .get(module)
                    .map(|p| p.pypath.as_str())
                    .unwrap(),
            );
        }
        Ok(packages)
    }

    pub fn child_packages(&self, package: &str) -> Result<HashSet<&str>> {
        let package: &Package = match self.borrow_packages_by_pypath().get(package) {
            Some(package) => package,
            None => {
                return Err(Error::PackageNotFound(package.to_string()))?;
            }
        };
        let mut packages = HashSet::new();
        for child in package.children.iter() {
            packages.insert(child.pypath.as_str());
        }
        Ok(packages)
    }

    pub fn child_modules(&self, package: &str) -> Result<HashSet<&str>> {
        let package: &Package = match self.borrow_packages_by_pypath().get(package) {
            Some(package) => package,
            None => {
                return Err(Error::PackageNotFound(package.to_string()))?;
            }
        };
        let mut modules = HashSet::new();
        for module in package.modules.iter() {
            modules.insert(module.pypath.as_str());
        }
        Ok(modules)
    }

    pub fn descendant_modules(&self, package: &str) -> Result<HashSet<&str>> {
        Ok(self
            ._descendant_modules(package)?
            .iter()
            .map(|m| m.pypath.as_str())
            .collect())
    }

    fn _descendant_modules(&self, package: &str) -> Result<HashSet<&Module>> {
        let package: &Package = match self.borrow_packages_by_pypath().get(package) {
            Some(package) => package,
            None => {
                return Err(Error::PackageNotFound(package.to_string()))?;
            }
        };
        let mut modules = HashSet::new();
        let mut q = vec![package];
        while let Some(package) = q.pop() {
            for module in package.modules.iter() {
                modules.insert(module);
            }
            for child in package.children.iter() {
                q.push(child);
            }
        }
        Ok(modules)
    }

    pub fn modules_directly_imported_by(&self, module_or_package: &str) -> Result<HashSet<&str>> {
        let from_modules = match self.borrow_packages_by_pypath().get(module_or_package) {
            Some(_) => self._descendant_modules(module_or_package)?,
            None => match self.borrow_modules_by_pypath().get(module_or_package) {
                Some(module) => HashSet::from([module as &Module]),
                None => Err(Error::ModuleNotFound(module_or_package.to_string()))?,
            },
        };
        let mut modules = HashSet::new();
        for from_module in from_modules {
            modules.extend(
                self.borrow_imports()
                    .get(from_module)
                    .unwrap()
                    .iter()
                    .map(|module| module.pypath.as_str())
                    .collect::<Vec<_>>(),
            )
        }
        Ok(modules)
    }

    pub fn modules_that_directly_import(&self, module_or_package: &str) -> Result<HashSet<&str>> {
        let to_modules = match self.borrow_packages_by_pypath().get(module_or_package) {
            Some(_) => self._descendant_modules(module_or_package)?,
            None => match self.borrow_modules_by_pypath().get(module_or_package) {
                Some(module) => HashSet::from([module as &Module]),
                None => Err(Error::ModuleNotFound(module_or_package.to_string()))?,
            },
        };
        let mut modules = HashSet::new();
        for to_module in to_modules {
            modules.extend(
                self.borrow_reverse_imports()
                    .get(to_module)
                    .unwrap()
                    .iter()
                    .map(|module| module.pypath.as_str())
                    .collect::<Vec<_>>(),
            )
        }
        Ok(modules)
    }

    pub fn downstream_modules(&self, module_or_package: &str) -> Result<HashSet<&str>> {
        let from_modules = match self.borrow_packages_by_pypath().get(module_or_package) {
            Some(_) => self._descendant_modules(module_or_package)?,
            None => match self.borrow_modules_by_pypath().get(module_or_package) {
                Some(module) => HashSet::from([module as &Module]),
                None => Err(Error::ModuleNotFound(module_or_package.to_string()))?,
            },
        };
        let mut downstream_modules = HashSet::new();
        for from_module in from_modules {
            let reachable_modules = bfs_reach(from_module, |module| {
                self.borrow_imports()
                    .get(module)
                    .unwrap()
                    .iter()
                    .map(|m| m as &Module)
                    .collect::<Vec<_>>()
            })
            .map(|m| m.pypath.as_str())
            .skip(1) // Remove starting module from the results.
            .collect::<Vec<_>>();
            downstream_modules.extend(reachable_modules);
        }
        Ok(downstream_modules)
    }

    pub fn upstream_modules(&self, module_or_package: &str) -> Result<HashSet<&str>> {
        let to_modules = match self.borrow_packages_by_pypath().get(module_or_package) {
            Some(_) => self._descendant_modules(module_or_package)?,
            None => match self.borrow_modules_by_pypath().get(module_or_package) {
                Some(module) => HashSet::from([module as &Module]),
                None => Err(Error::ModuleNotFound(module_or_package.to_string()))?,
            },
        };
        let mut upstream_modules = HashSet::new();
        for to_module in to_modules {
            let reachable_modules = bfs_reach(to_module, |module| {
                self.borrow_reverse_imports()
                    .get(module)
                    .unwrap()
                    .iter()
                    .map(|m| m as &Module)
                    .collect::<Vec<_>>()
            })
            .map(|m| m.pypath.as_str())
            .skip(1) // Remove starting module from the results.
            .collect::<Vec<_>>();
            upstream_modules.extend(reachable_modules);
        }
        Ok(upstream_modules)
    }

    pub fn path_exists(&self, from_module: &str, to_module: &str) -> Result<bool> {
        let from_module = match self.borrow_modules_by_pypath().get(from_module) {
            Some(module) => module as &Module,
            None => Err(Error::ModuleNotFound(from_module.to_string()))?,
        };
        let to_module = match self.borrow_modules_by_pypath().get(to_module) {
            Some(module) => module as &Module,
            None => Err(Error::ModuleNotFound(to_module.to_string()))?,
        };
        Ok(self._shortest_path(from_module, to_module).is_some())
    }

    pub fn shortest_path(&self, from_module: &str, to_module: &str) -> Result<Option<Vec<&str>>> {
        let from_module = match self.borrow_modules_by_pypath().get(from_module) {
            Some(module) => module as &Module,
            None => Err(Error::ModuleNotFound(from_module.to_string()))?,
        };
        let to_module = match self.borrow_modules_by_pypath().get(to_module) {
            Some(module) => module as &Module,
            None => Err(Error::ModuleNotFound(to_module.to_string()))?,
        };
        return Ok(self._shortest_path(from_module, to_module));
    }

    fn _shortest_path<'a>(
        &'a self,
        from_module: &'a Module,
        to_module: &'a Module,
    ) -> Option<Vec<&'a str>> {
        let shortest_path = bfs(
            &from_module,
            |module| {
                self.borrow_imports()
                    .get(module)
                    .unwrap()
                    .iter()
                    .map(|m| m as &Module)
                    .collect::<Vec<_>>()
            },
            |module| *module == to_module,
        );
        shortest_path.map(|shortest_path| shortest_path.iter().map(|m| m.pypath.as_str()).collect())
    }
}
