use std::collections::HashMap;
use crate::transpiler::ast::*;

#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipState {
    Active,
    Moved,
    Alias { source: String, is_mut: bool },
}

pub struct OwnershipChecker {
    scopes: Vec<HashMap<String, OwnershipState>>,
}

impl OwnershipChecker {
    pub fn new() -> Self {
        Self { scopes: vec![HashMap::new()] }
    }

    pub fn check(&mut self, ast: &[Node<Statement>]) -> Result<(), String> {
        for stmt in ast {
            self.check_statement(stmt)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, node: &Node<Statement>) -> Result<(), String> {
        let span = node.span;
        match &node.data {
            Statement::LetDeclaration(decl) => {
                if let Some(init) = &decl.init {
                    self.check_expression(init, span, true)?;
                }
                self.define_variable(&decl.name);
                Ok(())
            }
            Statement::UnsafeDeclaration(decl) => {
                if let Some(init) = &decl.init {
                    self.check_expression(init, span, false)?;
                }
                self.define_variable(&decl.name);
                Ok(())
            }
            Statement::AliasDeclaration(decl) => {
                self.ensure_active(&decl.source, span)?;
                self.scopes.last_mut().unwrap().insert(decl.name.clone(), OwnershipState::Alias { 
                    source: decl.source.clone(), 
                    is_mut: decl.is_mut 
                });
                Ok(())
            }
            Statement::Expression(expr) => self.check_expression(expr, span, false),
            Statement::Block(stmts) => {
                self.enter_scope();
                for s in stmts { self.check_statement(s)?; }
                self.exit_scope();
                Ok(())
            }
            Statement::If { condition, then_branch, else_branch } => {
                self.check_expression(condition, span, false)?;
                
                // For simplicity, we assume if either branch moves, the variable is moved
                // A better approach would be to branch the scope and merge states
                self.check_statement(then_branch)?;
                if let Some(eb) = else_branch {
                    self.check_statement(eb)?;
                }
                Ok(())
            }
            Statement::Return(Some(expr)) => self.check_expression(expr, span, true),
            Statement::FunctionDef(func) => {
                self.enter_scope();
                for (_ty, name) in &func.params { self.define_variable(name); }
                self.check_statement(&func.body)?;
                self.exit_scope();
                Ok(())
            }
            Statement::BindBlock(bind) => {
                for func in &bind.functions {
                    self.check_statement(&Node { data: Statement::FunctionDef(func.clone()), span })?;
                }
                Ok(())
            }
            Statement::ForkBlock(fork) => {
                for bop in &fork.bind_ops {
                    match bop {
                        BindOp::Add(f) | BindOp::Patch { new_fn_def: f, .. } => {
                            self.check_statement(&Node { data: Statement::FunctionDef(f.clone()), span })?;
                        }
                        _ => {}
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn check_expression(&mut self, expr: &Expression, span: Span, is_move_context: bool) -> Result<(), String> {
        match expr {
            Expression::Identifier(name) => {
                if is_move_context {
                    self.move_variable(name, span)
                } else {
                    self.ensure_active(name, span)
                }
            }
            Expression::MethodCall { receiver, method, args } => {
                if method == "clone" {
                    self.check_expression(receiver, span, false)?;
                } else {
                    self.check_expression(receiver, span, false)?;
                }
                for arg in args { self.check_expression(arg, span, is_move_context)?; }
                Ok(())
            }
            Expression::BinaryOp(l, op, r) => {
                self.check_expression(l, span, op == "=" || is_move_context)?;
                self.check_expression(r, span, is_move_context)?;
                Ok(())
            }
            Expression::MemberAccess { receiver, .. } => self.check_expression(receiver, span, false),
            Expression::Call(_, args) => {
                for arg in args { self.check_expression(arg, span, true)?; }
                Ok(())
            }
            Expression::Assignment(name, val) => {
                self.check_expression(val, span, true)?;
                self.define_variable(name); // Re-activate or define
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn define_variable(&mut self, name: &str) {
        self.scopes.last_mut().unwrap().insert(name.to_string(), OwnershipState::Active);
    }

    fn move_variable(&mut self, name: &str, span: Span) -> Result<(), String> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(state) = scope.get_mut(name) {
                if *state == OwnershipState::Moved {
                    return Err(format!("[{}:{}] Use of moved value: '{}'", span.line, span.col, name));
                }
                *state = OwnershipState::Moved;
                return Ok(())
            }
        }
        Ok(())
    }

    fn ensure_active(&self, name: &str, span: Span) -> Result<(), String> {
        for scope in self.scopes.iter().rev() {
            if let Some(state) = scope.get(name) {
                match state {
                    OwnershipState::Moved => return Err(format!("[{}:{}] Value '{}' was moved.", span.line, span.col, name)),
                    OwnershipState::Alias { source, .. } => return self.ensure_active(source, span),
                    _ => return Ok(()),
                }
            }
        }
        Ok(())
    }

    fn enter_scope(&mut self) { self.scopes.push(HashMap::new()); }
    fn exit_scope(&mut self) { self.scopes.pop(); }
}
