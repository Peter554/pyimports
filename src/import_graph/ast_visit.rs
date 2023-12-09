use anyhow::Result;
use rustpython_parser::ast::{ExceptHandler, ModModule, Stmt};

pub(super) enum VisitChildren {
    All,
    Some(Vec<Stmt>),
    None,
}

pub(super) trait StatementVisitor {
    fn visit(&mut self, stmt: &Stmt) -> VisitChildren;
}

pub(super) fn visit_statements<T>(ast: &ModModule, visitor: &mut T) -> Result<()>
where
    T: StatementVisitor,
{
    for stmt in ast.body.iter() {
        visit_stmt(stmt, visitor)?;
    }
    Ok(())
}

fn visit_stmt<T>(stmt: &Stmt, visitor: &mut T) -> Result<()>
where
    T: StatementVisitor,
{
    let visit_children = visitor.visit(stmt);
    if let VisitChildren::None = visit_children {
        // Pass.
    } else if let VisitChildren::All = visit_children {
        match stmt {
            Stmt::FunctionDef(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor)?;
                }
            }
            Stmt::AsyncFunctionDef(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor)?;
                }
            }
            Stmt::ClassDef(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor)?;
                }
            }
            Stmt::Return(_) => {}
            Stmt::Delete(_) => {}
            Stmt::Assign(_) => {}
            Stmt::TypeAlias(_) => {}
            Stmt::AugAssign(_) => {}
            Stmt::AnnAssign(_) => {}
            Stmt::For(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor)?;
                }
                for stmt in def.orelse.iter() {
                    visit_stmt(stmt, visitor)?;
                }
            }
            Stmt::AsyncFor(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor)?;
                }
                for stmt in def.orelse.iter() {
                    visit_stmt(stmt, visitor)?;
                }
            }
            Stmt::While(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor)?;
                }
                for stmt in def.orelse.iter() {
                    visit_stmt(stmt, visitor)?;
                }
            }
            Stmt::If(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor)?;
                }
                for stmt in def.orelse.iter() {
                    visit_stmt(stmt, visitor)?;
                }
            }
            Stmt::With(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor)?;
                }
            }
            Stmt::AsyncWith(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor)?;
                }
            }
            Stmt::Match(def) => {
                for case in def.cases.iter() {
                    for stmt in case.body.iter() {
                        visit_stmt(stmt, visitor)?;
                    }
                }
            }
            Stmt::Raise(_) => {}
            Stmt::Try(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor)?;
                }
                for handler in def.handlers.iter() {
                    match handler {
                        ExceptHandler::ExceptHandler(handler) => {
                            for stmt in handler.body.iter() {
                                visit_stmt(stmt, visitor)?;
                            }
                        }
                    }
                }
                for stmt in def.orelse.iter() {
                    visit_stmt(stmt, visitor)?;
                }
                for stmt in def.finalbody.iter() {
                    visit_stmt(stmt, visitor)?;
                }
            }
            Stmt::TryStar(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor)?;
                }
                for handler in def.handlers.iter() {
                    match handler {
                        ExceptHandler::ExceptHandler(handler) => {
                            for stmt in handler.body.iter() {
                                visit_stmt(stmt, visitor)?;
                            }
                        }
                    }
                }
                for stmt in def.orelse.iter() {
                    visit_stmt(stmt, visitor)?;
                }
                for stmt in def.finalbody.iter() {
                    visit_stmt(stmt, visitor)?;
                }
            }
            Stmt::Assert(_) => {}
            Stmt::Import(_) => {}
            Stmt::ImportFrom(_) => {}
            Stmt::Global(_) => {}
            Stmt::Nonlocal(_) => {}
            Stmt::Expr(_) => {}
            Stmt::Pass(_) => {}
            Stmt::Break(_) => {}
            Stmt::Continue(_) => {}
        }
    } else if let VisitChildren::Some(stmts) = visit_children {
        for stmt in stmts {
            visit_stmt(&stmt, visitor)?;
        }
    }
    Ok(())
}
