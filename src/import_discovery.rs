use anyhow::{bail, Result};
use rayon::prelude::*;
use rustpython_parser::{
    self,
    ast::{Mod, Stmt},
};
use std::collections::HashSet;
use std::fs;
use std::{collections::HashMap, sync::Arc};

use crate::ast_visit;
use crate::indexing;
use crate::package_discovery;

pub type Imports = HashMap<String, HashSet<String>>;

pub fn discover_imports(modules_by_pypath: &indexing::ModulesByPypath) -> Result<Imports> {
    modules_by_pypath
        .values()
        .par_bridge()
        .map(|module| {
            let imports = get_imports_for_module(Arc::clone(module), modules_by_pypath)?;
            Ok((module.pypath.clone(), imports))
        })
        .collect::<Result<Imports>>()
}

fn get_imports_for_module(
    module: Arc<package_discovery::Module>,
    modules_by_pypath: &indexing::ModulesByPypath,
) -> Result<HashSet<String>> {
    let code = fs::read_to_string(&module.path)?;
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
        module: Arc::clone(&module),
        modules_by_pypath,
        imports: HashSet::new(),
    };
    ast_visit::visit_statements(&ast, &mut visitor);

    Ok(visitor.imports)
}

struct ImportVisitor<'a> {
    module: Arc<package_discovery::Module>,
    modules_by_pypath: &'a indexing::ModulesByPypath,
    imports: HashSet<String>,
}

impl<'a> ast_visit::StatementVisitor for ImportVisitor<'a> {
    fn visit(&mut self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Import(stmt) => {
                for name in stmt.names.iter() {
                    for pypath in [name.name.to_string(), format!("{}.__init__", name.name)] {
                        match self.modules_by_pypath.get(&pypath) {
                            Some(imported_module) => {
                                self.imports.insert(imported_module.pypath.clone());
                            }
                            None => continue,
                        }
                    }
                }
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
                                self.module.pypath.split(".").collect::<Vec<_>>();
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

                for name in stmt.names.iter() {
                    for pypath in [
                        format!("{}.{}", &pypath_prefix, name.name),
                        pypath_prefix.clone(),
                        format!("{}.{}.__init__", &pypath_prefix, name.name),
                        format!("{}.__init__", &pypath_prefix),
                    ] {
                        match self.modules_by_pypath.get(&pypath) {
                            Some(imported_module) => {
                                self.imports.insert(imported_module.pypath.clone());
                                break;
                            }
                            None => continue,
                        }
                    }
                }
            }
            _ => {}
        }
        return true;
    }
}

#[cfg(test)]
mod tests {
    use crate::indexing;
    use std::{collections::HashSet, path::Path};

    use super::*;

    #[test]
    fn test_get_imports_for_module() {
        let root_package_path = Path::new("./example");
        let root_package = package_discovery::discover_package(root_package_path).unwrap();
        let modules_by_pypath = indexing::get_modules_by_pypath(Arc::clone(&root_package)).unwrap();

        let module = modules_by_pypath.get("example.__init__").unwrap();
        let imports = get_imports_for_module(Arc::clone(module), &modules_by_pypath).unwrap();
        assert_eq!(
            imports,
            [
                "example.a",
                "example.child.c_a",
                "example.b",
                "example.child.c_b",
                "example.c",
                "example.child.c_c",
                "example.d",
                "example.child.c_d",
                "example.e",
                "example.child.c_e",
                "example.child.__init__",
                "example.child2.__init__",
                "example.child3.__init__",
                "example.child4.__init__",
                "example.child5.__init__",
            ]
            .into_iter()
            .map(|i| i.to_string())
            .collect::<HashSet<_>>()
        );

        let module = modules_by_pypath.get("example.child.__init__").unwrap();
        let imports = get_imports_for_module(Arc::clone(module), &modules_by_pypath).unwrap();
        assert_eq!(
            imports,
            [
                "example.a",
                "example.child.c_a",
                "example.b",
                "example.child.c_b",
                "example.c",
                "example.child.c_c",
                "example.d",
                "example.child.c_d",
                "example.e",
                "example.child.c_e",
                "example.__init__",
                "example.child2.__init__",
                "example.child3.__init__",
                "example.child4.__init__",
                "example.child5.__init__",
            ]
            .into_iter()
            .map(|i| i.to_string())
            .collect::<HashSet<_>>()
        );

        let module = modules_by_pypath.get("example.z").unwrap();
        let imports = get_imports_for_module(Arc::clone(module), &modules_by_pypath).unwrap();
        assert_eq!(
            imports,
            [
                "example.a",
                "example.child.c_a",
                "example.b",
                "example.child.c_b",
                "example.c",
                "example.child.c_c",
                "example.d",
                "example.child.c_d",
                "example.e",
                "example.child.c_e",
                "example.child.__init__",
                "example.child2.__init__",
                "example.child3.__init__",
                "example.child4.__init__",
                "example.child5.__init__",
            ]
            .into_iter()
            .map(|i| i.to_string())
            .collect::<HashSet<_>>()
        );

        let module = modules_by_pypath.get("example.child.c_z").unwrap();
        let imports = get_imports_for_module(Arc::clone(module), &modules_by_pypath).unwrap();
        assert_eq!(
            imports,
            [
                "example.a",
                "example.child.c_a",
                "example.b",
                "example.child.c_b",
                "example.c",
                "example.child.c_c",
                "example.d",
                "example.child.c_d",
                "example.e",
                "example.child.c_e",
                "example.__init__",
                "example.child2.__init__",
                "example.child3.__init__",
                "example.child4.__init__",
                "example.child5.__init__",
            ]
            .into_iter()
            .map(|i| i.to_string())
            .collect::<HashSet<_>>()
        );
    }
}
