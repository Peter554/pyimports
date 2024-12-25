use anyhow::Result;
use rustpython_parser::{
    self,
    ast::{Mod, Stmt},
    source_code::LinearLocator,
};
use std::{fs, path::Path};

use crate::utils::path_to_pypath;
use crate::{errors::Error, import_discovery::ast_visit};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawImport {
    pub pypath: String,
    pub line_number: usize,
    pub is_typechecking: bool,
}

pub fn discover_imports(path: &Path) -> Result<Vec<RawImport>> {
    let code = fs::read_to_string(path)?;

    let ast = match rustpython_parser::parse(
        &code,
        rustpython_parser::Mode::Module,
        path.to_str().unwrap(),
    ) {
        Ok(ast) => ast,
        Err(err) => Err(Error::UnableToParsePythonFile {
            path: path.to_path_buf(),
            parse_error: err,
        })?,
    };

    let locator = LinearLocator::new(&code);

    let mut visitor = ImportVisitor {
        locator,
        imports: vec![],
    };

    ast_visit::visit_statements(
        &ast,
        &mut visitor,
        VisitorContext {
            is_typechecking: false,
        },
    )?;

    Ok(visitor.imports)
}

pub fn resolve_relative_imports(
    path: &Path,
    imports: Vec<RawImport>,
    root_path: &Path,
) -> Result<Vec<RawImport>> {
    let mut imports = imports.clone();
    for import in imports.iter_mut() {
        if import.pypath.starts_with(".") {
            let trimmed_pypath = import.pypath.trim_start_matches(".");
            let base_pypath = {
                let n = import.pypath.len() - trimmed_pypath.len();
                let mut base_path = path;
                for _ in 0..n {
                    base_path = base_path.parent().unwrap();
                }
                path_to_pypath(base_path, root_path).unwrap()
            };
            import.pypath = base_pypath + "." + trimmed_pypath;
        }
    }
    Ok(imports)
}

struct ImportVisitor<'a> {
    locator: LinearLocator<'a>,
    imports: Vec<RawImport>,
}

#[derive(Debug)]
struct VisitorContext {
    is_typechecking: bool,
}

impl ast_visit::StatementVisitor<VisitorContext> for ImportVisitor<'_> {
    fn visit(
        &mut self,
        stmt: &Stmt,
        context: &VisitorContext,
    ) -> ast_visit::VisitChildren<VisitorContext> {
        match stmt {
            Stmt::Import(stmt) => {
                for name in stmt.names.iter() {
                    let location = self.locator.locate(name.range.start());
                    self.imports.push(RawImport {
                        pypath: name.name.to_string(),
                        line_number: location.row.to_usize(),
                        is_typechecking: context.is_typechecking,
                    });
                }
                ast_visit::VisitChildren::None
            }
            Stmt::ImportFrom(stmt) => {
                let mut prefix = String::new();

                if let Some(level) = &stmt.level {
                    prefix += &".".repeat(level.to_usize())
                }

                if let Some(module) = &stmt.module {
                    prefix += module.clone().as_ref();
                    prefix += ".";
                }

                for name in stmt.names.iter() {
                    let location = self.locator.locate(name.range.start());
                    self.imports.push(RawImport {
                        pypath: prefix.clone() + name.name.as_ref(),
                        line_number: location.row.to_usize(),
                        is_typechecking: context.is_typechecking,
                    });
                }
                ast_visit::VisitChildren::None
            }
            Stmt::If(stmt) => {
                let mut is_typechecking_if = false;
                if stmt.test.is_attribute_expr() {
                    let expression = stmt.test.clone().expect_attribute_expr();
                    is_typechecking_if = expression.attr.as_str() == "TYPE_CHECKING";
                } else if stmt.test.is_name_expr() {
                    let expression = stmt.test.clone().expect_name_expr();
                    is_typechecking_if = expression.id.as_str() == "TYPE_CHECKING";
                }

                if is_typechecking_if {
                    ast_visit::VisitChildren::Some(vec![
                        (
                            VisitorContext {
                                is_typechecking: true,
                            },
                            stmt.body.clone(),
                        ),
                        (
                            VisitorContext {
                                is_typechecking: false,
                            },
                            stmt.orelse.clone(),
                        ),
                    ])
                } else {
                    ast_visit::VisitChildren::All
                }
            }
            _ => ast_visit::VisitChildren::All,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutils::{testpackage, TestPackage};
    use maplit::hashmap;
    use parameterized::parameterized;
    use std::collections::HashSet;

    struct TestCase<'a> {
        code: &'a str,
        expected_imports: Vec<RawImport>,
    }

    #[parameterized(case={
        TestCase {
            code: "",
            expected_imports: vec![]
        },
        TestCase {
            code: "import foo",
            expected_imports: vec![
                RawImport {pypath: "foo".into(), line_number: 1, is_typechecking: false}
            ]
        },
        TestCase {
            code: "import foo as FOO",
            expected_imports: vec![
                RawImport {pypath: "foo".into(), line_number: 1, is_typechecking: false}
            ]
        },
        TestCase {
            code: "import foo, bar",
            expected_imports: vec![
                RawImport {pypath: "foo".into(), line_number: 1, is_typechecking: false},
                RawImport {pypath: "bar".into(), line_number: 1, is_typechecking: false}
            ]
        },
        TestCase {
            code: "
import foo
import bar",
            expected_imports: vec![
                RawImport {pypath: "foo".into(), line_number: 2, is_typechecking: false},
                RawImport {pypath: "bar".into(), line_number: 3, is_typechecking: false}
            ]
        },
        TestCase {
            code: "import foo.bar",
            expected_imports: vec![
                RawImport {pypath: "foo.bar".into(), line_number: 1, is_typechecking: false}
            ]
        },
        TestCase {
            code: "from foo import bar",
            expected_imports: vec![
                RawImport {pypath: "foo.bar".into(), line_number: 1, is_typechecking: false}
            ]
        },
        TestCase {
            code: "from foo import bar as BAR",
            expected_imports: vec![
                RawImport {pypath: "foo.bar".into(), line_number: 1, is_typechecking: false}
            ]
        },
        TestCase {
            code: "from foo import bar, baz",
            expected_imports: vec![
                RawImport {pypath: "foo.bar".into(), line_number: 1, is_typechecking: false},
                RawImport {pypath: "foo.baz".into(), line_number: 1, is_typechecking: false},
            ]
        },
        TestCase {
            code: "from . import foo",
            expected_imports: vec![
                RawImport {pypath: ".foo".into(), line_number: 1, is_typechecking: false}
            ]
        },
        TestCase {
            code: "from .foo import bar",
            expected_imports: vec![
                RawImport {pypath: ".foo.bar".into(), line_number: 1, is_typechecking: false}
            ]
        },
        TestCase {
            code: "from .. import foo",
            expected_imports: vec![
                RawImport {pypath: "..foo".into(), line_number: 1, is_typechecking: false}
            ]
        },
        TestCase {
            code: "from ..foo import bar",
            expected_imports: vec![
                RawImport {pypath: "..foo.bar".into(), line_number: 1, is_typechecking: false}
            ]
        },
        TestCase {
            code: "from ..foo.bar import baz",
            expected_imports: vec![
                RawImport {pypath: "..foo.bar.baz".into(), line_number: 1, is_typechecking: false}
            ]
        },
        TestCase {
            code: "
def f():
    import foo",
            expected_imports: vec![
                RawImport {pypath: "foo".into(), line_number: 3, is_typechecking: false}
            ]
        },
        TestCase {
            code: "
import typing

if typing.TYPE_CHECKING:
    import foo
else:
    import bar",
            expected_imports: vec![
                RawImport {pypath: "typing".into(), line_number: 2, is_typechecking: false},
                RawImport {pypath: "foo".into(), line_number: 5, is_typechecking: true},
                RawImport {pypath: "bar".into(), line_number: 7, is_typechecking: false} 
            ]
        },
        TestCase {
            code: "
import typing as t

if t.TYPE_CHECKING:
    import foo
else:
    import bar",
            expected_imports: vec![
                RawImport {pypath: "typing".into(), line_number: 2, is_typechecking: false},
                RawImport {pypath: "foo".into(), line_number: 5, is_typechecking: true},
                RawImport {pypath: "bar".into(), line_number: 7, is_typechecking: false} 
            ]
        },
        TestCase {
            code: "
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import foo
else:
    import bar",
            expected_imports: vec![
                RawImport {pypath: "typing.TYPE_CHECKING".into(), line_number: 2, is_typechecking: false},
                RawImport {pypath: "foo".into(), line_number: 5, is_typechecking: true},
                RawImport {pypath: "bar".into(), line_number: 7, is_typechecking: false}
            ]
        },
    })]
    fn test_discover_imports(case: TestCase) -> Result<()> {
        let test_package = testpackage! {
            "__init__.py" => case.code
        };

        let imports = discover_imports(&test_package.path().join("__init__.py"))?;

        let imports = imports.into_iter().collect::<Vec<_>>();

        assert_eq!(imports.len(), case.expected_imports.len());
        assert_eq!(
            imports.into_iter().collect::<HashSet<_>>(),
            case.expected_imports.into_iter().collect::<HashSet<_>>()
        );

        Ok(())
    }

    struct RelativeImportsTestCase<'a> {
        path: &'a str,
        code: &'a str,
        expected_imports: Vec<RawImport>,
    }

    #[parameterized(case={
        RelativeImportsTestCase {
            path: "foo.py",
            code: "from . import bar",
            expected_imports: vec![
                RawImport {pypath: "testpackage.bar".into(), line_number: 1, is_typechecking: false}
            ]
        },
        RelativeImportsTestCase {
            path: "subpackage/foo.py",
            code: "from .. import bar",
            expected_imports: vec![
                RawImport {pypath: "testpackage.bar".into(), line_number: 1, is_typechecking: false}
            ]
        },
        RelativeImportsTestCase {
            path: "foo.py",
            code: "from .bar import ABC",
            expected_imports: vec![
                RawImport {pypath: "testpackage.bar.ABC".into(), line_number: 1, is_typechecking: false}
            ]
        },
        RelativeImportsTestCase {
            path: "subpackage/foo.py",
            code: "from ..bar import ABC",
            expected_imports: vec![
                RawImport {pypath: "testpackage.bar.ABC".into(), line_number: 1, is_typechecking: false}
            ]
        },
    })]
    fn test_resolve_relative_imports(case: RelativeImportsTestCase<'_>) -> Result<()> {
        let test_package = testpackage! {
                "__init__.py" => "",
                "bar.py" => "",
                case.path => case.code
        };

        let imports = discover_imports(&test_package.path().join(case.path))?;

        let imports = resolve_relative_imports(
            &test_package.path().join(case.path),
            imports,
            test_package.path(),
        )?;

        assert_eq!(imports.len(), case.expected_imports.len());
        assert_eq!(
            imports.into_iter().collect::<HashSet<_>>(),
            case.expected_imports.into_iter().collect::<HashSet<_>>()
        );

        Ok(())
    }
}
