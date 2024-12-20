use anyhow::Result;
use rustpython_parser::ast::{ExceptHandler, ModModule, Stmt};

pub fn visit_statements<TVisitor, TContext>(
    ast: &ModModule,
    visitor: &mut TVisitor,
    context: TContext,
) -> Result<()>
where
    TVisitor: StatementVisitor<TContext>,
{
    for stmt in ast.body.iter() {
        visit_stmt(stmt, visitor, &context)?;
    }
    Ok(())
}

pub trait StatementVisitor<TContext> {
    fn visit(&mut self, stmt: &Stmt, context: &TContext) -> VisitChildren<TContext>;
}

pub enum VisitChildren<TContext> {
    All,
    None,
    Some(Vec<(TContext, Vec<Stmt>)>),
}

fn visit_stmt<TVisitor, TContext>(
    stmt: &Stmt,
    visitor: &mut TVisitor,
    context: &TContext,
) -> Result<()>
where
    TVisitor: StatementVisitor<TContext>,
{
    let visit_children = visitor.visit(stmt, context);
    if let VisitChildren::None = visit_children {
        // Pass.
    } else if let VisitChildren::All = visit_children {
        match stmt {
            Stmt::FunctionDef(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor, context)?;
                }
            }
            Stmt::AsyncFunctionDef(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor, context)?;
                }
            }
            Stmt::ClassDef(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor, context)?;
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
                    visit_stmt(stmt, visitor, context)?;
                }
                for stmt in def.orelse.iter() {
                    visit_stmt(stmt, visitor, context)?;
                }
            }
            Stmt::AsyncFor(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor, context)?;
                }
                for stmt in def.orelse.iter() {
                    visit_stmt(stmt, visitor, context)?;
                }
            }
            Stmt::While(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor, context)?;
                }
                for stmt in def.orelse.iter() {
                    visit_stmt(stmt, visitor, context)?;
                }
            }
            Stmt::If(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor, context)?;
                }
                for stmt in def.orelse.iter() {
                    visit_stmt(stmt, visitor, context)?;
                }
            }
            Stmt::With(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor, context)?;
                }
            }
            Stmt::AsyncWith(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor, context)?;
                }
            }
            Stmt::Match(def) => {
                for case in def.cases.iter() {
                    for stmt in case.body.iter() {
                        visit_stmt(stmt, visitor, context)?;
                    }
                }
            }
            Stmt::Raise(_) => {}
            Stmt::Try(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor, context)?;
                }
                for handler in def.handlers.iter() {
                    match handler {
                        ExceptHandler::ExceptHandler(handler) => {
                            for stmt in handler.body.iter() {
                                visit_stmt(stmt, visitor, context)?;
                            }
                        }
                    }
                }
                for stmt in def.orelse.iter() {
                    visit_stmt(stmt, visitor, context)?;
                }
                for stmt in def.finalbody.iter() {
                    visit_stmt(stmt, visitor, context)?;
                }
            }
            Stmt::TryStar(def) => {
                for stmt in def.body.iter() {
                    visit_stmt(stmt, visitor, context)?;
                }
                for handler in def.handlers.iter() {
                    match handler {
                        ExceptHandler::ExceptHandler(handler) => {
                            for stmt in handler.body.iter() {
                                visit_stmt(stmt, visitor, context)?;
                            }
                        }
                    }
                }
                for stmt in def.orelse.iter() {
                    visit_stmt(stmt, visitor, context)?;
                }
                for stmt in def.finalbody.iter() {
                    visit_stmt(stmt, visitor, context)?;
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
    } else if let VisitChildren::Some(ctx_stmts) = visit_children {
        for (context, stmts) in ctx_stmts {
            for stmt in stmts {
                visit_stmt(&stmt, visitor, &context)?;
            }
        }
    } else {
        panic!()
    }
    Ok(())
}
