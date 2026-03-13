use std::collections::HashMap;
use crate::transpiler::ast::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorrowState {
    None,
    Immutable(usize),
    Mutable,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OwnershipStatus {
    Available,
    Moved,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableMeta {
    pub status: OwnershipStatus,
    pub borrows: BorrowState,
    pub is_alias: bool,
    pub alias_source: Option<String>,
    pub is_mut_alias: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccessContext {
    Move,
    Read,
    MutBorrow,
    ImmBorrow,
    Write,
}

pub struct OwnershipChecker {
    scopes: Vec<HashMap<String, VariableMeta>>,
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
                    self.check_expression(init, AccessContext::Move)?;
                }
                self.define_variable(&decl.name, false, None, false);
                Ok(())
            }
            Statement::UnsafeDeclaration(decl) => {
                if let Some(init) = &decl.init {
                    self.check_expression(init, AccessContext::Read)?;
                }
                self.define_variable(&decl.name, false, None, false);
                Ok(())
            }
            Statement::AliasDeclaration(decl) => {
                // 1. 验证源变量是否可借用
                let context = if decl.is_mut { AccessContext::MutBorrow } else { AccessContext::ImmBorrow };
                let dummy_node = Node { data: Expression::Identifier(decl.source.clone()), span };
                self.check_expression(&dummy_node, context)?;
                
                // 2. 更新源变量借用状态
                self.apply_borrow(&decl.source, decl.is_mut)?;

                // 3. 定义别名变量
                self.define_variable(&decl.name, true, Some(decl.source.clone()), decl.is_mut);
                Ok(())
            }
            Statement::Expression(expr) => self.check_expression(expr, AccessContext::Read),
            Statement::Block(stmts) => {
                self.enter_scope();
                for s in stmts { self.check_statement(s)?; }
                self.exit_scope();
                Ok(())
            }
            Statement::If { condition, then_branch, else_branch } => {
                self.check_expression(condition, AccessContext::Read)?;
                
                // 捕获当前作用域状态以进行分支合并
                let original_scope = self.scopes.last().unwrap().clone();
                
                self.check_statement(then_branch)?;
                let scope_after_then = self.scopes.last().unwrap().clone();
                
                // 重置到 if 前的状态检查 else
                *self.scopes.last_mut().unwrap() = original_scope;
                
                if let Some(eb) = else_branch {
                    self.check_statement(eb)?;
                    let scope_after_else = self.scopes.last().unwrap().clone();
                    self.merge_scopes(scope_after_then, scope_after_else);
                } else {
                    self.merge_scopes(scope_after_then, self.scopes.last().unwrap().clone());
                }
                Ok(())
            }
            Statement::Return(Some(expr)) => self.check_expression(expr, AccessContext::Move),
            Statement::FunctionDef(func) => {
                self.enter_scope();
                for (_ty, name) in &func.params { self.define_variable(name, false, None, false); }
                self.check_statement(&func.body)?;
                self.exit_scope();
                Ok(())
            }
            Statement::BindBlock(bind) => {
                for func in &bind.functions {
                    self.enter_scope();
                    // 定义 host 变量
                    self.define_variable("host", false, None, false);
                    for (_ty, name) in &func.params { self.define_variable(name, false, None, false); }
                    self.check_statement(&func.body)?;
                    self.exit_scope();
                }
                Ok(())
            }
            Statement::ForkBlock(fork) => {
                for bop in &fork.bind_ops {
                    match bop {
                        BindOp::Add(f) | BindOp::Patch { new_fn_def: f, .. } => {
                            self.enter_scope();
                            // 定义 host 变量
                            self.define_variable("host", false, None, false);
                            for (_ty, name) in &f.params { self.define_variable(name, false, None, false); }
                            self.check_statement(&f.body)?;
                            self.exit_scope();
                        }
                        _ => {}
                    }
                }
                Ok(())
            }
            Statement::Spawn(_func, args) => {
                for arg in args {
                    self.check_expression(arg, AccessContext::Move)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn check_expression(&mut self, expr_node: &Node<Expression>, context: AccessContext) -> Result<(), String> {
        let span = expr_node.span;
        match &expr_node.data {
            Expression::Identifier(name) => self.validate_access(name, context, span),
            Expression::MethodCall { receiver, method, args } => {
                if method == "clone" {
                    self.check_expression(receiver, AccessContext::Read)?;
                } else {
                    // 方法调用通常视为可变借用（简单化处理）
                    self.check_expression(receiver, AccessContext::MutBorrow)?;
                }
                for arg in args { self.check_expression(arg, AccessContext::Move)?; }
                Ok(())
            }
            Expression::BinaryOp(l, op, r) => {
                if op == "=" {
                    self.check_expression(l, AccessContext::Write)?;
                    self.check_expression(r, AccessContext::Move)?;
                } else {
                    self.check_expression(l, AccessContext::Read)?;
                    self.check_expression(r, AccessContext::Read)?;
                }
                Ok(())
            }
            Expression::MemberAccess { receiver, .. } => self.check_expression(receiver, AccessContext::Read),
            Expression::Call(_, args) => {
                for arg in args { self.check_expression(arg, AccessContext::Move)?; }
                Ok(())
            }
            Expression::Assignment(name, val) => {
                self.check_expression(val, AccessContext::Move)?;
                self.define_variable(name, false, None, false); // 重新激活变量
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn validate_access(&mut self, name: &str, context: AccessContext, span: Span) -> Result<(), String> {
        let meta = self.find_variable(name).ok_or_else(|| format!("[{}:{}] Undefined variable '{}'", span.line, span.col, name))?;

        if meta.status == OwnershipStatus::Moved {
            return Err(format!("[{}:{}] Use of moved value: '{}'", span.line, span.col, name));
        }

        match context {
            AccessContext::Move => {
                if meta.borrows != BorrowState::None {
                    return Err(format!("[{}:{}] Cannot move borrowed value: '{}'", span.line, span.col, name));
                }
                self.mark_moved(name);
            }
            AccessContext::MutBorrow => {
                if meta.borrows != BorrowState::None {
                    return Err(format!("[{}:{}] Cannot mut-borrow value with active borrows: '{}'", span.line, span.col, name));
                }
            }
            AccessContext::ImmBorrow => {
                if meta.borrows == BorrowState::Mutable {
                    return Err(format!("[{}:{}] Cannot imm-borrow mutably borrowed value: '{}'", span.line, span.col, name));
                }
            }
            AccessContext::Write => {
                if meta.borrows != BorrowState::None {
                    return Err(format!("[{}:{}] Cannot assign to borrowed value: '{}'", span.line, span.col, name));
                }
            }
            AccessContext::Read => {
                // 读取始终允许，只要不是 Moved
            }
        }
        Ok(())
    }

    fn define_variable(&mut self, name: &str, is_alias: bool, source: Option<String>, is_mut: bool) {
        let meta = VariableMeta {
            status: OwnershipStatus::Available,
            borrows: BorrowState::None,
            is_alias,
            alias_source: source,
            is_mut_alias: is_mut,
        };
        self.scopes.last_mut().unwrap().insert(name.to_string(), meta);
    }

    fn apply_borrow(&mut self, name: &str, is_mut: bool) -> Result<(), String> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(meta) = scope.get_mut(name) {
                meta.borrows = match (meta.borrows, is_mut) {
                    (BorrowState::None, false) => BorrowState::Immutable(1),
                    (BorrowState::None, true) => BorrowState::Mutable,
                    (BorrowState::Immutable(n), false) => BorrowState::Immutable(n + 1),
                    _ => return Err(format!("Internal Error: Borrow conflict not caught for '{}'", name)),
                };
                return Ok(());
            }
        }
        Ok(())
    }

    fn mark_moved(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(meta) = scope.get_mut(name) {
                meta.status = OwnershipStatus::Moved;
                return;
            }
        }
    }

    fn find_variable(&self, name: &str) -> Option<VariableMeta> {
        for scope in self.scopes.iter().rev() {
            if let Some(meta) = scope.get(name) {
                return Some(meta.clone());
            }
        }
        None
    }

    fn enter_scope(&mut self) { self.scopes.push(HashMap::new()); }

    fn exit_scope(&mut self) {
        let current_scope = self.scopes.pop().unwrap();
        // 释放当前作用域别名对源变量的借用
        for (_, meta) in current_scope {
            if meta.is_alias {
                if let Some(source) = meta.alias_source {
                    self.release_borrow(&source, meta.is_mut_alias);
                }
            }
        }
    }

    fn release_borrow(&mut self, name: &str, is_mut: bool) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(meta) = scope.get_mut(name) {
                meta.borrows = match (meta.borrows, is_mut) {
                    (BorrowState::Immutable(n), false) if n > 1 => BorrowState::Immutable(n - 1),
                    (BorrowState::Immutable(1), false) => BorrowState::None,
                    (BorrowState::Mutable, true) => BorrowState::None,
                    (state, _) => state, // Should not happen in well-formed code
                };
                return;
            }
        }
    }

    fn merge_scopes(&mut self, then_scope: HashMap<String, VariableMeta>, else_scope: HashMap<String, VariableMeta>) {
        let current = self.scopes.last_mut().unwrap();
        for (name, meta) in current.iter_mut() {
            let in_then = then_scope.get(name);
            let in_else = else_scope.get(name);
            
            match (in_then, in_else) {
                (Some(t), Some(e)) => {
                    if t.status == OwnershipStatus::Moved || e.status == OwnershipStatus::Moved {
                        meta.status = OwnershipStatus::Moved;
                    }
                }
                (Some(t), None) => {
                    if t.status == OwnershipStatus::Moved { meta.status = OwnershipStatus::Moved; }
                }
                _ => {}
            }
        }
    }
}
