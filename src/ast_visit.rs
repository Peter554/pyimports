use rustpython_parser::ast::{ExceptHandler, ModModule, Stmt};

pub trait StatementVisitor {
    fn visit(&mut self, stmt: &Stmt) -> bool;
}

pub fn visit_statements<T>(ast: &ModModule, visitor: &mut T)
where
    T: StatementVisitor,
{
    for stmt in ast.body.iter() {
        visit_stmt(stmt, visitor);
    }
}

fn visit_stmt<T>(stmt: &Stmt, visitor: &mut T)
where
    T: StatementVisitor,
{
    let ok = visitor.visit(stmt);
    if !ok {
        return;
    }
    match stmt {
        Stmt::FunctionDef(def) => {
            for stmt in def.body.iter() {
                visit_stmt(&stmt, visitor);
            }
        }
        Stmt::AsyncFunctionDef(def) => {
            for stmt in def.body.iter() {
                visit_stmt(&stmt, visitor);
            }
        }
        Stmt::ClassDef(def) => {
            for stmt in def.body.iter() {
                visit_stmt(&stmt, visitor);
            }
        }
        Stmt::Return(_) => return,
        Stmt::Delete(_) => return,
        Stmt::Assign(_) => return,
        Stmt::TypeAlias(_) => return,
        Stmt::AugAssign(_) => return,
        Stmt::AnnAssign(_) => return,
        Stmt::For(def) => {
            for stmt in def.body.iter() {
                visit_stmt(&stmt, visitor);
            }
            for stmt in def.orelse.iter() {
                visit_stmt(&stmt, visitor);
            }
        }
        Stmt::AsyncFor(def) => {
            for stmt in def.body.iter() {
                visit_stmt(&stmt, visitor);
            }
            for stmt in def.orelse.iter() {
                visit_stmt(&stmt, visitor);
            }
        }
        Stmt::While(def) => {
            for stmt in def.body.iter() {
                visit_stmt(&stmt, visitor);
            }
            for stmt in def.orelse.iter() {
                visit_stmt(&stmt, visitor);
            }
        }
        Stmt::If(def) => {
            for stmt in def.body.iter() {
                visit_stmt(&stmt, visitor);
            }
            for stmt in def.orelse.iter() {
                visit_stmt(&stmt, visitor);
            }
        }
        Stmt::With(def) => {
            for stmt in def.body.iter() {
                visit_stmt(&stmt, visitor);
            }
        }
        Stmt::AsyncWith(def) => {
            for stmt in def.body.iter() {
                visit_stmt(&stmt, visitor);
            }
        }
        Stmt::Match(def) => {
            for case in def.cases.iter() {
                for stmt in case.body.iter() {
                    visit_stmt(&stmt, visitor);
                }
            }
        }
        Stmt::Raise(_) => return,
        Stmt::Try(def) => {
            for stmt in def.body.iter() {
                visit_stmt(&stmt, visitor);
            }
            for handler in def.handlers.iter() {
                match handler {
                    ExceptHandler::ExceptHandler(handler) => {
                        for stmt in handler.body.iter() {
                            visit_stmt(&stmt, visitor);
                        }
                    }
                }
            }
            for stmt in def.orelse.iter() {
                visit_stmt(&stmt, visitor);
            }
            for stmt in def.finalbody.iter() {
                visit_stmt(&stmt, visitor);
            }
        }
        Stmt::TryStar(def) => {
            for stmt in def.body.iter() {
                visit_stmt(&stmt, visitor);
            }
            for handler in def.handlers.iter() {
                match handler {
                    ExceptHandler::ExceptHandler(handler) => {
                        for stmt in handler.body.iter() {
                            visit_stmt(&stmt, visitor);
                        }
                    }
                }
            }
            for stmt in def.orelse.iter() {
                visit_stmt(&stmt, visitor);
            }
            for stmt in def.finalbody.iter() {
                visit_stmt(&stmt, visitor);
            }
        }
        Stmt::Assert(_) => return,
        Stmt::Import(_) => return,
        Stmt::ImportFrom(_) => return,
        Stmt::Global(_) => return,
        Stmt::Nonlocal(_) => return,
        Stmt::Expr(_) => return,
        Stmt::Pass(_) => return,
        Stmt::Break(_) => return,
        Stmt::Continue(_) => return,
    }
}

#[cfg(test)]
mod tests {
    use rustpython_parser::ast::Mod;
    use std::{fs, path::Path};

    use super::*;

    struct TestVisitor {
        counter: u64,
    }

    impl StatementVisitor for TestVisitor {
        fn visit(&mut self, stmt: &Stmt) -> bool {
            self.counter += 1;
            return true;
        }
    }

    #[test]
    fn test_visit() {
        let path = Path::new("./src/ast_visit_test.py");
        let code = fs::read_to_string(path).unwrap();
        let parse_result = rustpython_parser::parse(
            &code,
            rustpython_parser::Mode::Module,
            path.to_str().unwrap(),
        );
        let ast = match parse_result {
            Ok(m) => match m {
                Mod::Module(mm) => mm,
                _ => panic!(),
            },
            _ => panic!(),
        };

        let mut visitor = TestVisitor { counter: 0 };

        visit_statements(&ast, &mut visitor);

        assert_eq!(visitor.counter, 5);
    }
}
