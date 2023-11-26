use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::graph;
use super::import_discovery;
use super::indexing;
use super::package_discovery;

pub struct ImportGraphBuilder {
    pub(super) root_package_path: PathBuf, // ignored_imports
                                           // ignore_type_checking_imports
                                           // cache_config
}

impl ImportGraphBuilder {
    pub fn new(root_package_path: &Path) -> Self {
        ImportGraphBuilder {
            root_package_path: root_package_path.to_path_buf(),
        }
    }

    pub fn build(&self) -> Result<graph::ImportGraph> {
        let root_package = package_discovery::discover_package(self.root_package_path.as_path())?;
        let packages_by_pypath = indexing::get_packages_by_pypath(Arc::clone(&root_package))?;
        let modules_by_pypath = indexing::get_modules_by_pypath(Arc::clone(&root_package))?;
        let packages_by_module = indexing::get_packages_by_module(Arc::clone(&root_package))?;
        let imports =
            import_discovery::discover_imports(Arc::clone(&root_package), &modules_by_pypath)?;
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
