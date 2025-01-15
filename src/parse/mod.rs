//! The `parse` module provides functionality to parse
//! the import statements from a single python file.

mod ast_visit;

use crate::errors::Error;
use crate::pypath::Pypath;
use anyhow::Result;
use derive_new::new;
use getset::{CopyGetters, Getters};
use rustpython_parser::{self, ast::Stmt, source_code::LinearLocator};
use std::{fs, path::Path};
use tap::Conv;

/// An import within a python file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, new, Getters, CopyGetters)]
pub struct RawImport {
    /// The imported pypath. Can be absolute or relative.
    #[new(into)]
    #[getset(get = "pub")]
    pypath: String,
    /// The line number of the import.
    #[getset(get_copy = "pub")]
    line_number: usize,
    /// Whether the import is `TYPE_CHECKING`.
    /// This is determined as a best guess by inspecting the AST for statements of the form:
    /// - `if TYPE_CHECKING:`
    /// - `if xxx.TYPE_CHECKING:`
    #[getset(get_copy = "pub")]
    is_typechecking: bool,
}

/// Parses the python file at the passed filesystem path and returns a vector of discovered imports.
///
/// ```
/// # use anyhow::Result;
/// # use pyimports::{testpackage, testutils::TestPackage};
/// use pyimports::parse::{parse_imports,RawImport};
///
/// # fn main() -> Result<()> {
/// let testpackage = testpackage! {
///     "__init__.py" => "
/// import typing
/// import testpackage.foo
/// from testpackage import bar
///
/// if typing.TYPE_CHECKING:
///     from . import baz
/// "
/// };
///
/// let imports = parse_imports(&testpackage.path().join("__init__.py"))?;
/// assert_eq!(
///     imports,
///     vec![
///         RawImport::new("typing", 2, false),
///         RawImport::new("testpackage.foo", 3, false),
///         RawImport::new("testpackage.bar", 4, false),
///         RawImport::new(".baz", 7, true),
///     ]
/// );
/// # Ok(())
/// # }
/// ```
pub fn parse_imports(path: &Path) -> Result<Vec<RawImport>> {
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

/// Resolves relative and wildcard imports.
///
/// # Relative import
///
/// ```
/// # use anyhow::Result;
/// # use pyimports::{testpackage, testutils::TestPackage};
/// use pyimports::parse::resolve_import;
///
/// # fn main() -> Result<()> {
/// let testpackage = testpackage! {
///     "__init__.py" => "from . import foo",
///     "foo.py" => ""
/// };
///
/// let pypath = resolve_import(
///     ".foo",
///     &testpackage.path().join("__init__.py"),
///     &testpackage.path()
/// )?;
/// assert_eq!(pypath, "testpackage.foo".parse()?);
/// # Ok(())
/// # }
/// ```
///
/// # Wildcard import
///
/// ```
/// # use anyhow::Result;
/// # use pyimports::{testpackage, testutils::TestPackage};
/// use pyimports::parse::resolve_import;
///
/// # fn main() -> Result<()> {
/// let testpackage = testpackage! {
///     "__init__.py" => "from foo import *",
///     "foo.py" => ""
/// };
///
/// let pypath = resolve_import(
///     "testpackage.foo.*",
///     &testpackage.path().join("__init__.py"),
///     &testpackage.path()
/// )?;
/// assert_eq!(pypath, "testpackage.foo".parse()?);
/// # Ok(())
/// # }
/// ```
pub fn resolve_import(
    imported_pypath: &str,
    module_path: &Path,
    root_path: &Path,
) -> Result<Pypath> {
    let mut imported_pypath = imported_pypath.to_string();

    if imported_pypath.ends_with(".*") {
        imported_pypath = imported_pypath.strip_suffix(".*").unwrap().to_owned()
    }

    if !imported_pypath.starts_with(".") {
        return Ok(imported_pypath.parse()?);
    }

    let trimmed_pypath = imported_pypath.trim_start_matches(".");
    let base_pypath = {
        let n = imported_pypath.len() - trimmed_pypath.len();
        let mut base_path = module_path;
        for _ in 0..n {
            base_path = base_path.parent().unwrap();
        }
        Pypath::from_path(base_path, root_path).unwrap()
    };
    Ok((base_pypath.conv::<String>() + "." + trimmed_pypath).parse()?)
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
    use crate::{testpackage, testutils::TestPackage};

    use parameterized::parameterized;
    use std::{collections::HashSet, path::PathBuf};

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
            code: "from foo import *",
            expected_imports: vec![
                RawImport {pypath: "foo.*".into(), line_number: 1, is_typechecking: false},
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
    fn test_parse_imports(case: TestCase) -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => case.code
        };

        let imports = parse_imports(&testpackage.path().join("__init__.py"))?;

        let imports = imports.into_iter().collect::<Vec<_>>();

        assert_eq!(imports.len(), case.expected_imports.len());
        assert_eq!(
            imports.into_iter().collect::<HashSet<_>>(),
            case.expected_imports.into_iter().collect::<HashSet<_>>()
        );

        Ok(())
    }

    struct RelativeImportsTestCase<'a> {
        pypath: &'a str,
        path: &'a str,
        expected: Pypath,
    }

    #[parameterized(case={
        RelativeImportsTestCase {
            pypath: ".bar",
            path: "foo.py",
            expected:  Pypath::new("testpackage.bar")
        },
        RelativeImportsTestCase {
            pypath: "..bar",
            path: "subpackage/foo.py",
            expected:   Pypath::new("testpackage.bar")
        },
        RelativeImportsTestCase {
            pypath: ".bar.ABC",
            path: "foo.py",
            expected: Pypath::new("testpackage.bar.ABC")
        },
        RelativeImportsTestCase {
            pypath: "..bar.ABC",
            path: "subpackage/foo.py",
            expected:   Pypath::new("testpackage.bar.ABC")
        },
        RelativeImportsTestCase {
            pypath: ".bar.*",
            path: "foo.py",
            expected:  Pypath::new("testpackage.bar")
        },
    })]
    fn test_resolve_import(case: RelativeImportsTestCase<'_>) -> Result<()> {
        let testpackage = testpackage! {
            "__init__.py" => ""
        };

        assert_eq!(
            resolve_import(
                case.pypath,
                &testpackage.path().join(PathBuf::from(case.path)),
                testpackage.path()
            )?,
            case.expected
        );

        Ok(())
    }
}
