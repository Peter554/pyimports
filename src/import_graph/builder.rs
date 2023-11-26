use anyhow::Result;
use std::path::{Path, PathBuf};

use super::graph;

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

    pub fn build(self) -> Result<graph::ImportGraph> {
        graph::ImportGraph::build(self)
    }
}
