use crate::transpiler::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipStatus {
    Active,
    Moved,
    Alias { is_mut: bool },
}

pub struct Generator {
    output: String,
    indent_level: usize,
    struct_registry: HashMap<String, StructDef>,
    scope_vars: Vec<HashMap<String, (String, OwnershipStatus)>>, 
    defined_destructors: Vec<String>,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            struct_registry: HashMap::new(),
            scope_vars: vec![HashMap::new()],
            defined_destructors: Vec::new(),
        }
    }

    pub fn generate(&mut self, ast: &[Node<Statement>]) -> String {
        self.output.push_str("#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n#include <stdbool.h>\n\n");
        
        for node in ast {
            self.pre_register(&node.data);
        }

        let mut structs_to_gen = Vec::new();
        for node in ast {
            if let Statement::StructDef(def) = &node.data {
                structs_to_gen.push(def.clone());
            } else if let Statement::ForkBlock(fork) = &node.data {
                if let Some(new_def) = self.struct_registry.get(&fork.new_name).cloned() {
                    structs_to_gen.push(new_def);
                }
            }
        }

        for def in structs_to_gen {
            self.generate_struct_def(&def);
        }

        for node in ast {
            match &node.data {
                Statement::StructDef(_) => {},
                Statement::ForkBlock(fork) => {
                    for bop in &fork.bind_ops {
                        match bop {
                            BindOp::Add(func) | BindOp::Patch { new_fn_def: func, .. } => {
                                self.generate_bound_function(&fork.new_name, func);
                            }
                            _ => {}
                        }
                    }
                }
                _ => self.generate_statement(node),
            }
        }

        self.output.clone()
    }

    fn pre_register(&mut self, stmt: &Statement) {
        match stmt {
            Statement::StructDef(def) => {
                self.struct_registry.insert(def.name.clone(), def.clone());
            }
            Statement::BindBlock(bind) => {
                for func in &bind.functions {
                    if func.name == "destroy" {
                        self.defined_destructors.push(bind.struct_name.clone());
                    }
                }
            }
            Statement::ForkBlock(fork) => {
                if let Some(base) = self.struct_registry.get(&fork.base_name).cloned() {
                    let mut new_fields = base.fields;
                    for op in &fork.operations {
                        match op {
                            ForkOp::Add(field) => new_fields.push(field.clone()),
                            ForkOp::Remove(name) => new_fields.retain(|f| &f.name != name),
                        }
                    }
                    let new_def = StructDef { name: fork.new_name.clone(), fields: new_fields };
                    self.struct_registry.insert(new_def.name.clone(), new_def.clone());
                    
                    for bop in &fork.bind_ops {
                        match bop {
                            BindOp::Add(func) | BindOp::Patch { new_fn_def: func, .. } => {
                                if func.name == "destroy" {
                                    self.defined_destructors.push(fork.new_name.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn generate_struct_def(&mut self, def: &StructDef) {
        self.output.push_str(&format!("typedef struct {} {{\n", def.name));
        for field in &def.fields {
            self.output.push_str(&format!("    {} {};\n", field.ty, field.name));
        }
        self.output.push_str(&format!("}} {};\n\n", def.name));

        self.output.push_str(&format!("{} {}_clone(const {}* source) {{\n", def.name, def.name, def.name));
        self.output.push_str(&format!("    {} dest;\n", def.name));
        for field in &def.fields {
            if self.struct_registry.contains_key(&field.ty) {
                self.output.push_str(&format!("    dest.{} = {}_clone(&source->{});\n", field.name, field.ty, field.name));
            } else {
                self.output.push_str(&format!("    dest.{} = source->{};\n", field.name, field.name));
            }
        }
        self.output.push_str("    return dest;\n");
        self.output.push_str("}\n\n");

        self.output.push_str(&format!("void {}_init_defaults({}* host) {{\n", def.name, def.name));
        for field in &def.fields {
            if let Some(val) = &field.default_value {
                let translated = self.translate_expr(val);
                self.output.push_str(&format!("    host->{} = {};\n", field.name, translated));
            } else {
                self.output.push_str(&format!("    memset(&host->{}, 0, sizeof({}));\n", field.name, field.ty));
            }
        }
        self.output.push_str("}\n\n");
    }

    fn generate_bound_function(&mut self, struct_name: &str, func: &FunctionDef) {
        let is_constructor = func.name == struct_name;
        let is_destructor = func.name == "destroy";
        let c_func_name = if is_constructor {
            format!("{}_init", struct_name)
        } else if is_destructor {
            format!("{}_destroy", struct_name)
        } else {
            format!("{}_{}", struct_name, func.name)
        };

        self.output.push_str(&format!("{} {}({}* host", func.return_ty, c_func_name, struct_name));
        for (ty, name) in &func.params {
            self.output.push_str(&format!(", {} {}", ty, name));
        }
        self.output.push_str(") ");
        
        self.scope_vars.push(HashMap::new());
        self.scope_vars.last_mut().unwrap().insert("host".to_string(), (format!("{}*", struct_name), OwnershipStatus::Active));
        for (ty, name) in &func.params {
             self.scope_vars.last_mut().unwrap().insert(name.clone(), (ty.clone(), OwnershipStatus::Active));
        }

        self.generate_statement(&func.body);
        self.scope_vars.pop();
        self.output.push_str("\n\n");
    }

    fn generate_statement(&mut self, node: &Node<Statement>) {
        match &node.data {
            Statement::StructDef(def) => self.generate_struct_def(def),
            Statement::BindBlock(bind) => {
                for func in &bind.functions {
                    self.generate_bound_function(&bind.struct_name, func);
                }
            }
            Statement::FunctionDef(func) => {
                self.output.push_str(&format!("{} {}(", func.return_ty, func.name));
                for (i, (ty, name)) in func.params.iter().enumerate() {
                    if i > 0 { self.output.push_str(", "); }
                    self.output.push_str(&format!("{} {}", ty, name));
                }
                self.output.push_str(") ");
                
                self.scope_vars.push(HashMap::new());
                for (ty, name) in &func.params {
                    self.scope_vars.last_mut().unwrap().insert(name.clone(), (ty.clone(), OwnershipStatus::Active));
                }
                self.generate_statement(&func.body);
                self.scope_vars.pop();
                self.output.push_str("\n\n");
            }
            Statement::Block(stmts) => {
                self.output.push_str("{\n");
                self.indent_level += 1;
                self.scope_vars.push(HashMap::new());
                for s in stmts { self.generate_statement(s); }
                
                let current_scope = self.scope_vars.pop().unwrap();
                for (name, (ty, status)) in current_scope {
                    if status == OwnershipStatus::Active && self.defined_destructors.contains(&ty) {
                        self.push_indent();
                        self.output.push_str(&format!("{}_destroy(&{});\n", ty, name));
                    }
                }
                
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}");
            }
            Statement::LetDeclaration(decl) | Statement::UnsafeDeclaration(decl) => {
                self.push_indent();
                self.output.push_str(&format!("{} {} ", decl.ty, decl.name));
                if let Some(init) = &decl.init {
                    let translated = self.translate_expr(init);
                    self.output.push_str(&format!("= {};\n", translated));
                    if let Expression::Identifier(name) = init {
                        self.mark_moved(name);
                    }
                } else {
                    self.output.push_str("= {0};\n");
                }
                
                if let Some(scope) = self.scope_vars.last_mut() {
                    scope.insert(decl.name.clone(), (decl.ty.clone(), OwnershipStatus::Active));
                }
                
                if let Some(call) = &decl.constructor_call {
                    self.push_indent();
                    self.output.push_str(&format!("{}_init_defaults(&{});\n", decl.ty, decl.name));
                    self.push_indent();
                    self.output.push_str(&format!("{}_init(&{}", decl.ty, decl.name));
                    for arg in &call.args {
                        let arg_translated = self.translate_expr(arg);
                        self.output.push_str(&format!(", {}", arg_translated));
                        if let Expression::Identifier(name) = arg {
                            self.mark_moved(name);
                        }
                    }
                    self.output.push_str(");\n");
                }
            }
            Statement::AliasDeclaration(decl) => {
                self.push_indent();
                if decl.is_mut {
                    self.output.push_str(&format!("{}* {} = &{};\n", decl.ty, decl.name, decl.source));
                } else {
                    self.output.push_str(&format!("const {}* {} = &{};\n", decl.ty, decl.name, decl.source));
                }
                if let Some(scope) = self.scope_vars.last_mut() {
                    scope.insert(decl.name.clone(), (decl.ty.clone(), OwnershipStatus::Alias { is_mut: decl.is_mut }));
                }
            }
            Statement::Return(expr) => {
                self.push_indent();
                self.output.push_str("return");
                if let Some(e) = expr {
                    let translated = self.translate_expr(e);
                    self.output.push_str(&format!(" {}", translated));
                    if let Expression::Identifier(name) = e {
                        self.mark_moved(name);
                    }
                }
                self.output.push_str(";\n");
            }
            Statement::Expression(expr) => {
                self.push_indent();
                let translated = self.translate_expr(expr);
                self.output.push_str(&translated);
                self.output.push_str(";\n");
            }
            Statement::If { condition, then_branch, else_branch } => {
                self.push_indent();
                let cond_translated = self.translate_expr(condition);
                self.output.push_str(&format!("if ({}) ", cond_translated));
                self.generate_statement(then_branch);
                if let Some(eb) = else_branch {
                    self.output.push_str(" else ");
                    self.generate_statement(eb);
                }
                self.output.push_str("\n");
            }
            Statement::RawC(c) => {
                self.output.push_str(c);
                self.output.push_str("\n");
            }
            Statement::ForkBlock(_) => {},
        }
    }

    fn translate_expr(&mut self, expr: &Expression) -> String {
        match expr {
            Expression::Identifier(name) => name.clone(),
            Expression::Number(n) => n.clone(),
            Expression::StringLiteral(s) => format!("\"{}\"", s),
            Expression::BinaryOp(left, op, right) => {
                let l_str = self.translate_expr(left);
                let r_str = self.translate_expr(right);
                if op == "" {
                    return format!("{} {}", l_str, r_str);
                }
                if op == "=" {
                    if let Expression::Identifier(name) = right.as_ref() {
                        self.mark_moved(name);
                    }
                }
                format!("({}) {} ({})", l_str, op, r_str)
            }
            Expression::Assignment(name, val) => {
                let val_str = self.translate_expr(val);
                if let Expression::Identifier(vname) = val.as_ref() {
                    self.mark_moved(vname);
                }
                format!("{} = {}", name, val_str)
            }
            Expression::Call(name, args) => {
                let mut arg_strs = Vec::new();
                for arg in args {
                    let arg_str = self.translate_expr(arg);
                    if let Expression::Identifier(aname) = arg {
                        self.mark_moved(aname);
                    }
                    arg_strs.push(arg_str);
                }
                format!("{}({})", name, arg_strs.join(", "))
            }
            Expression::MethodCall { receiver, method, args } => {
                let (obj_ty, is_ptr) = self.get_type_info(receiver);
                let receiver_raw = self.translate_expr(receiver);
                
                if method == "clone" {
                    if is_ptr {
                        return format!("{}_clone({})", obj_ty, receiver_raw);
                    } else {
                        return format!("{}_clone(&{})", obj_ty, receiver_raw);
                    }
                }

                let mut arg_strs = Vec::new();
                if is_ptr {
                    arg_strs.push(receiver_raw);
                } else {
                    arg_strs.push(format!("&{}", receiver_raw));
                }
                for arg in args {
                    let arg_str = self.translate_expr(arg);
                    if let Expression::Identifier(aname) = arg {
                        self.mark_moved(aname);
                    }
                    arg_strs.push(arg_str);
                }
                format!("{}_{}({})", obj_ty, method, arg_strs.join(", "))
            }
            Expression::MemberAccess { receiver, member, is_ptr: force_ptr } => {
                let (_, is_ptr) = self.get_type_info(receiver);
                let receiver_raw = self.translate_expr(receiver);
                let op = if is_ptr || *force_ptr { "->" } else { "." };
                format!("{}{}{}", receiver_raw, op, member)
            }
        }
    }

    fn get_type_info(&self, expr: &Expression) -> (String, bool) {
        match expr {
            Expression::Identifier(name) => {
                for scope in self.scope_vars.iter().rev() {
                    if let Some((ty, status)) = scope.get(name) {
                        let is_ptr = ty.ends_with('*') || matches!(status, OwnershipStatus::Alias { .. });
                        return (ty.trim_end_matches('*').to_string(), is_ptr);
                    }
                }
            }
            Expression::MemberAccess { receiver, member, .. } => {
                let (base_ty, _) = self.get_type_info(receiver);
                if let Some(def) = self.struct_registry.get(&base_ty) {
                    for field in &def.fields {
                        if &field.name == member {
                            let is_ptr = field.ty.ends_with('*');
                            return (field.ty.trim_end_matches('*').to_string(), is_ptr);
                        }
                    }
                }
            }
            _ => {}
        }
        ("Unknown".to_string(), false)
    }

    fn mark_moved(&mut self, name: &str) {
        for scope in self.scope_vars.iter_mut().rev() {
            if let Some(var) = scope.get_mut(name) {
                var.1 = OwnershipStatus::Moved;
                break;
            }
        }
    }

    fn push_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }
}
