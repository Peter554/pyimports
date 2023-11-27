use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::cache::{FileCache, ImportsCache, NullCache};
use super::graph;
use super::import_discovery;
use super::indexing;
use super::package_discovery;

pub struct ImportGraphBuilder {
    pub(super) root_package_path: PathBuf,
    pub(super) use_cache: bool,
}

impl ImportGraphBuilder {
    pub fn new(root_package_path: &Path) -> Self {
        ImportGraphBuilder {
            root_package_path: root_package_path.to_path_buf(),
            use_cache: false,
        }
    }

    pub fn use_cache(mut self) -> Self {
        self.use_cache = true;
        self
    }

    pub fn build(&self) -> Result<graph::ImportGraph> {
        let root_package = package_discovery::discover_package(self.root_package_path.as_path())?;
        let packages_by_pypath = indexing::get_packages_by_pypath(Arc::clone(&root_package))?;
        let modules_by_pypath = indexing::get_modules_by_pypath(Arc::clone(&root_package))?;
        let packages_by_module = indexing::get_packages_by_module(Arc::clone(&root_package))?;

        let mut cache: Box<dyn ImportsCache + Send + Sync> = if self.use_cache {
            Box::new(FileCache::open(
                self.root_package_path.as_path(),
                &modules_by_pypath,
            )?)
        } else {
            Box::new(NullCache)
        };
        let imports = import_discovery::discover_imports(
            Arc::clone(&root_package),
            &modules_by_pypath,
            cache.as_mut(),
        )?;
        let reverse_imports = indexing::reverse_imports(&imports)?;

        Ok(graph::ImportGraph {
            packages_by_pypath,
            modules_by_pypath,
            packages_by_module,
            imports,
            reverse_imports,
        })
    }
}
