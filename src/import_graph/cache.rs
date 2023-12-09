use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, create_dir_all},
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use super::{indexing::ModulesByPypath, package_discovery::Module};

pub(super) trait ImportsCache {
    fn get_imports(&self, module: &Arc<Module>) -> Option<HashSet<Arc<Module>>>;

    fn set_imports(&mut self, module: &Arc<Module>, imported_modules: &HashSet<Arc<Module>>);

    fn persist(&self) -> Result<()>;
}

pub(super) struct FileCache {
    modules_by_pypath: ModulesByPypath,
    pypaths_by_module: HashMap<Arc<Module>, Arc<String>>,
    file_dir: PathBuf,
    file_path: PathBuf,
    file_data: FileData,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileData {
    module_imports: HashMap<String, ModuleImports>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModuleImports {
    computed_at: u64,
    imports: HashSet<String>,
}

impl FileCache {
    pub(super) fn open(
        root_package_path: &Path,
        modules_by_pypath: &ModulesByPypath,
        exclude_type_checking_imports: bool,
    ) -> Result<Self> {
        let pypaths_by_module = modules_by_pypath
            .iter()
            .map(|(k, v)| (Arc::clone(v), Arc::clone(k)))
            .collect();

        let file_dir = root_package_path.join(".pyimports_cache");
        let file_path = file_dir.join(format!(
            "exclude_type_checking_imports={}",
            exclude_type_checking_imports
        ));
        let file_data: FileData = if file_path.exists() {
            let file_contents = fs::read_to_string(&file_path)?;
            serde_json::from_str(&file_contents)?
        } else {
            FileData {
                module_imports: HashMap::new(),
            }
        };

        Ok(FileCache {
            modules_by_pypath: modules_by_pypath.clone(),
            pypaths_by_module,
            file_dir,
            file_path,
            file_data,
        })
    }
}

impl ImportsCache for FileCache {
    fn get_imports(&self, module: &Arc<Module>) -> Option<HashSet<Arc<Module>>> {
        let pypath = self.pypaths_by_module.get(module)?.to_string();
        let module_imports = self.file_data.module_imports.get(&pypath)?;
        if module.mtime > module_imports.computed_at {
            return None;
        }
        Some(
            module_imports
                .imports
                .iter()
                .map(|pypath| Arc::clone(self.modules_by_pypath.get(pypath).unwrap()))
                .collect(),
        )
    }

    fn set_imports(&mut self, module: &Arc<Module>, imported_modules: &HashSet<Arc<Module>>) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.file_data.module_imports.insert(
            module.pypath.to_string(),
            ModuleImports {
                computed_at: now,
                imports: imported_modules
                    .iter()
                    .map(|m| m.pypath.to_string())
                    .collect(),
            },
        );
    }

    fn persist(&self) -> Result<()> {
        create_dir_all(&self.file_dir)?;
        let s = serde_json::to_string(&self.file_data)?;
        fs::write(&self.file_path, s)?;
        Ok(())
    }
}

pub(super) struct NullCache;

impl ImportsCache for NullCache {
    fn get_imports(&self, _module: &Arc<Module>) -> Option<HashSet<Arc<Module>>> {
        None
    }

    fn set_imports(&mut self, _module: &Arc<Module>, _imported_modules: &HashSet<Arc<Module>>) {}

    fn persist(&self) -> Result<()> {
        Ok(())
    }
}
