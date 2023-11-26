use anyhow::{bail, Result};
use rayon::prelude::*;
use rustpython_parser::{
    self,
    ast::{Mod, Stmt},
};
use std::collections::HashSet;
use std::fs;
use std::{collections::HashMap, sync::Arc};

use super::ast_visit;
use super::indexing;
use super::package_discovery::{Module, Package};

pub type Imports = HashMap<Arc<Module>, HashSet<Arc<Module>>>;

pub fn discover_imports(
    root_package: Arc<Package>,
    modules_by_pypath: &indexing::ModulesByPypath,
) -> Result<Imports> {
    modules_by_pypath
        .values()
        .par_bridge()
        .map(|module| {
            let imports = get_imports_for_module(
                Arc::clone(&root_package),
                Arc::clone(module),
                modules_by_pypath,
            )?;
            Ok((Arc::clone(module), imports))
        })
        .collect::<Result<Imports>>()
}

fn get_imports_for_module(
    root_package: Arc<Package>,
    module: Arc<Module>,
    modules_by_pypath: &indexing::ModulesByPypath,
) -> Result<HashSet<Arc<Module>>> {
    let code = fs::read_to_string(module.path.as_ref())?;
    let ast = rustpython_parser::parse(
        &code,
        rustpython_parser::Mode::Module,
        module.path.to_str().unwrap(),
    );
    let ast = match ast {
        Ok(m) => match m {
            Mod::Module(mm) => mm,
            _ => bail!("not a module"),
        },
        Err(e) => return Err(e)?,
    };

    let mut visitor = ImportVisitor {
        root_package,
        module,
        modules_by_pypath,
        imports: HashSet::new(),
    };
    ast_visit::visit_statements(&ast, &mut visitor)?;

    Ok(visitor.imports)
}

struct ImportVisitor<'a> {
    root_package: Arc<Package>,
    module: Arc<Module>,
    modules_by_pypath: &'a indexing::ModulesByPypath,
    imports: HashSet<Arc<Module>>,
}

impl ast_visit::StatementVisitor for ImportVisitor<'_> {
    fn visit(&mut self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Import(stmt) => {
                for name in stmt.names.iter() {
                    if !(name.name.as_str() == self.root_package.pypath.as_ref()
                        || name
                            .name
                            .as_str()
                            .starts_with((self.root_package.pypath.to_string() + ".").as_str()))
                    {
                        // An external import.
                        break;
                    }

                    let mut found_module = false;
                    for pypath in [name.name.to_string(), format!("{}.__init__", name.name)] {
                        match self.modules_by_pypath.get(&pypath) {
                            Some(imported_module) => {
                                self.imports.insert(Arc::clone(imported_module));
                                found_module = true;
                                break;
                            }
                            None => continue,
                        }
                    }
                    if !found_module {
                        panic!("Failed to find internal import {}", name.name)
                    }
                }
                false
            }
            Stmt::ImportFrom(stmt) => {
                let level_pypath_prefix = match stmt.level {
                    Some(ref level) => {
                        let level = level.to_usize();
                        if level == 0 {
                            // Absolute import.
                            String::default()
                        } else {
                            // Relative import.
                            let mut this_module_pypath_parts =
                                self.module.pypath.split('.').collect::<Vec<_>>();
                            this_module_pypath_parts.pop();
                            let len = this_module_pypath_parts.len();
                            this_module_pypath_parts
                                .into_iter()
                                .take(len + 1 - level)
                                .collect::<Vec<_>>()
                                .join(".")
                        }
                    }
                    None => String::default(),
                };

                let module_pypath_prefix = match stmt.module {
                    Some(ref module) => module.to_string(),
                    None => String::default(),
                };

                let pypath_prefix = match (level_pypath_prefix.len(), module_pypath_prefix.len()) {
                    (0, 0) => panic!("Could not parse import"),
                    (0, _) => module_pypath_prefix,
                    (_, 0) => level_pypath_prefix,
                    _ => format!("{}.{}", level_pypath_prefix, module_pypath_prefix),
                };

                if !(pypath_prefix.as_str() == self.root_package.pypath.as_ref()
                    || pypath_prefix
                        .starts_with((self.root_package.pypath.to_string() + ".").as_str()))
                {
                    // An external import.
                    return false;
                }

                for name in stmt.names.iter() {
                    let mut found_module = false;
                    for pypath in [
                        format!("{}.{}", &pypath_prefix, name.name),
                        pypath_prefix.clone(),
                        format!("{}.{}.__init__", &pypath_prefix, name.name),
                        format!("{}.__init__", &pypath_prefix),
                    ] {
                        match self.modules_by_pypath.get(&pypath) {
                            Some(imported_module) => {
                                self.imports.insert(Arc::clone(imported_module));
                                found_module = true;
                                break;
                            }
                            None => continue,
                        }
                    }
                    if !found_module {
                        panic!("Failed to find internal import {}", name.name)
                    }
                }
                false
            }
            _ => true,
        }
    }
}
