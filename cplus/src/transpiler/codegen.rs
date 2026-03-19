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
    spawn_decls: String,
    spawn_funcs: String,
    indent_level: usize,
    struct_registry: HashMap<String, StructDef>,
    bind_registry: HashMap<String, Vec<FunctionDef>>,
    scope_vars: Vec<HashMap<String, (String, OwnershipStatus)>>, 
    defined_destructors: Vec<String>,
    spawn_counter: usize,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            spawn_decls: String::new(),
            spawn_funcs: String::new(),
            indent_level: 0,
            struct_registry: HashMap::new(),
            bind_registry: HashMap::new(),
            scope_vars: vec![HashMap::new()],
            defined_destructors: Vec::new(),
            spawn_counter: 0,
        }
    }

    pub fn generate(&mut self, ast: &[Node<Statement>]) -> String {
        self.output.push_str("#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n#include <stdbool.h>\n#include <pthread.h>\n\n");
        
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

        // 占位符用于插入 spawn 相关的声明
        self.output.push_str("/*SPAWN_DECLS*/\n\n");

        for node in ast {
            match &node.data {
                Statement::StructDef(_) => {},
                Statement::ForkBlock(fork) => {
                    if let Some(funcs) = self.bind_registry.get(&fork.new_name).cloned() {
                        for func in funcs {
                            self.generate_bound_function(&fork.new_name, &func);
                        }
                    }
                }
                _ => self.generate_statement(node),
            }
        }

        let mut final_output = self.output.clone();
        final_output = final_output.replace("/*SPAWN_DECLS*/", &self.spawn_decls);
        final_output.push_str("\n");
        final_output.push_str(&self.spawn_funcs);

        final_output
    }

    fn pre_register(&mut self, stmt: &Statement) {
        match stmt {
            Statement::StructDef(def) => {
                self.struct_registry.insert(def.name.clone(), def.clone());
            }
            Statement::BindBlock(bind) => {
                self.bind_registry.insert(bind.struct_name.clone(), bind.functions.clone());
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
                    
                    // Clone functions from base
                    let mut new_funcs = Vec::new();
                    if let Some(base_funcs) = self.bind_registry.get(&fork.base_name) {
                        for mut f in base_funcs.clone() {
                            // If it's a constructor, rename it
                            if f.name == fork.base_name {
                                f.name = fork.new_name.clone();
                            }
                            new_funcs.push(f);
                        }
                    }

                    // Apply bind operations
                    for bop in &fork.bind_ops {
                        match bop {
                            BindOp::Add(func) => {
                                // If adding a function with existing name, it's an override/add
                                new_funcs.retain(|f| f.name != func.name);
                                new_funcs.push(func.clone());
                            }
                            BindOp::Remove(name) => {
                                new_funcs.retain(|f| &f.name != name);
                            }
                            BindOp::Patch { _old_fn_name: old, new_fn_def: func } => {
                                // Patch means replace or add
                                new_funcs.retain(|f| &f.name != old);
                                new_funcs.push(func.clone());
                            }
                        }
                    }

                    self.bind_registry.insert(fork.new_name.clone(), new_funcs.clone());

                    for func in &new_funcs {
                        if func.name == "destroy" {
                            if !self.defined_destructors.contains(&fork.new_name) {
                                self.defined_destructors.push(fork.new_name.clone());
                            }
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
                    if let Expression::Identifier(name) = &init.data {
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
                        if let Expression::Identifier(name) = &arg.data {
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
                let mut to_destroy = Vec::new();
                for scope in self.scope_vars.iter().rev() {
                    for (name, (ty, status)) in scope {
                        if *status == OwnershipStatus::Active && self.defined_destructors.contains(ty) {
                            to_destroy.push((name.clone(), ty.clone()));
                        }
                    }
                }
                
                for (name, ty) in to_destroy {
                    self.push_indent();
                    self.output.push_str(&format!("{}_destroy(&{});\n", ty, name));
                    self.mark_moved(&name);
                }

                self.push_indent();
                self.output.push_str("return");
                if let Some(e) = expr {
                    let translated = self.translate_expr(e);
                    self.output.push_str(&format!(" {}", translated));
                    if let Expression::Identifier(name) = &e.data {
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
            Statement::For { init, condition, step, body } => {
                self.push_indent();
                self.output.push_str("for (");
                if let Some(stmt) = init {
                    match &stmt.data {
                        Statement::LetDeclaration(decl) | Statement::UnsafeDeclaration(decl) => {
                            self.output.push_str(&format!("{} {} ", decl.ty, decl.name));
                            if let Some(expr) = &decl.init {
                                let val = self.translate_expr(expr);
                                self.output.push_str(&format!("= {}", val));
                            }
                            if let Some(scope) = self.scope_vars.last_mut() {
                                scope.insert(decl.name.clone(), (decl.ty.clone(), OwnershipStatus::Active));
                            }
                        }
                        Statement::Expression(expr) => {
                            let val = self.translate_expr(expr);
                            self.output.push_str(&val);
                        }
                        _ => {}
                    }
                }
                self.output.push_str("; ");
                if let Some(cond) = condition {
                    let val = self.translate_expr(cond);
                    self.output.push_str(&val);
                }
                self.output.push_str("; ");
                if let Some(s) = step {
                    let val = self.translate_expr(s);
                    self.output.push_str(&val);
                }
                self.output.push_str(") ");
                self.generate_statement(body);
                self.output.push_str("\n");
            }
            Statement::While { condition, body } => {
                self.push_indent();
                let cond_translated = self.translate_expr(condition);
                self.output.push_str(&format!("while ({}) ", cond_translated));
                self.generate_statement(body);
                self.output.push_str("\n");
            }
            Statement::RawC(c) => {
                self.output.push_str(c);
                self.output.push_str("\n");
            }
            Statement::Spawn(func, args) => self.generate_spawn(func, args),
            Statement::ForkBlock(_) => {},
        }
    }

    fn generate_spawn(&mut self, func: &str, args: &[Node<Expression>]) {
        let id = self.spawn_counter;
        self.spawn_counter += 1;
        
        let mut arg_infos = Vec::new();
        for arg in args {
            let (ty, is_ptr) = self.get_type_info(arg);
            let mut final_ty = ty;
            if final_ty == "Unknown" {
                 if let Expression::Identifier(name) = &arg.data {
                     final_ty = name.clone();
                 }
            }
            arg_infos.push((final_ty, is_ptr, self.translate_expr(arg)));
        }

        let struct_name = format!("_spawn_args_{}", id);
        let wrapper_name = format!("_spawn_wrapper_{}", id);

        self.spawn_decls.push_str(&format!("typedef struct {} {{\n", struct_name));
        for (i, (ty, is_ptr, _)) in arg_infos.iter().enumerate() {
            let ptr_mark = if *is_ptr { "*" } else { "" };
            self.spawn_decls.push_str(&format!("    {}{} arg{};\n", ty, ptr_mark, i));
        }
        self.spawn_decls.push_str(&format!("}} {};\n", struct_name));
        self.spawn_decls.push_str(&format!("void* {}(void* raw_arg);\n\n", wrapper_name));
        
        let mut wrapper = format!("void* {}(void* raw_arg) {{\n", wrapper_name);
        wrapper.push_str(&format!("    {}* args = ({}*)raw_arg;\n", struct_name, struct_name));
        
        let mut call_args = Vec::new();
        for (i, (ty, is_ptr, _)) in arg_infos.iter().enumerate() {
            wrapper.push_str(&format!("    {}{} val{} = args->arg{};\n", ty, if *is_ptr { "*" } else { "" }, i, i));
            call_args.push(format!("val{}", i));
        }
        wrapper.push_str("    free(args);\n\n");
        wrapper.push_str(&format!("    {}({});\n\n", func, call_args.join(", ")));
        
        for (i, (ty, is_ptr, _)) in arg_infos.iter().enumerate() {
            if self.defined_destructors.contains(ty) {
                if *is_ptr { wrapper.push_str(&format!("    {}_destroy(val{});\n", ty, i)); }
                else { wrapper.push_str(&format!("    {}_destroy(&val{});\n", ty, i)); }
            }
        }
        wrapper.push_str("    return NULL;\n}\n\n");
        self.spawn_funcs.push_str(&wrapper);

        self.push_indent();
        self.output.push_str("{\n");
        self.indent_level += 1;
        self.push_indent();
        self.output.push_str(&format!("{}* spawn_args = malloc(sizeof({}));\n", struct_name, struct_name));
        for (i, (_, _, val_str)) in arg_infos.iter().enumerate() {
            self.push_indent();
            self.output.push_str(&format!("spawn_args->arg{} = {};\n", i, val_str));
            if let Expression::Identifier(name) = &args[i].data {
                self.mark_moved(name);
            }
        }
        self.push_indent();
        self.output.push_str("pthread_t thread_id;\n");
        self.push_indent();
        self.output.push_str(&format!("pthread_create(&thread_id, NULL, {}, spawn_args);\n", wrapper_name));
        self.push_indent();
        self.output.push_str("pthread_detach(thread_id);\n");
        self.indent_level -= 1;
        self.push_indent();
        self.output.push_str("}\n");
    }

    fn translate_receiver(&mut self, expr: &Node<Expression>) -> String {
        match &expr.data {
            Expression::Identifier(name) => name.clone(),
            _ => self.translate_expr(expr),
        }
    }

    fn translate_expr(&mut self, expr: &Node<Expression>) -> String {
        match &expr.data {
            Expression::Identifier(name) => {
                name.clone()
            },
            Expression::Number(n) => n.clone(),
            Expression::StringLiteral(s) => format!("\"{}\"", s),
            Expression::BinaryOp(left, op, right) => {
                let l_str = self.translate_expr(left);
                let r_str = self.translate_expr(right);
                if op == " " { return format!("{} {}", l_str, r_str); }
                if op == "=" || op == "+=" || op == "-=" {
                    if let Expression::Identifier(name) = &right.data { self.mark_moved(name); }
                    return format!("{} {} {}", l_str, op, r_str);
                }
                format!("({}) {} ({})", l_str, op, r_str)
            }
            Expression::Assignment(name, val) => {
                let val_str = self.translate_expr(val);
                if let Expression::Identifier(vname) = &val.data { self.mark_moved(vname); }
                format!("{} = {}", name, val_str)
            }
            Expression::UnaryOp(op, expr, is_postfix) => {
                let expr_str = self.translate_expr(expr);
                if *is_postfix { format!("{}{}", expr_str, op) }
                else { format!("{}{}", op, expr_str) }
            }
            Expression::Call(name, args) => {
                let mut arg_strs = Vec::new();
                for arg in args {
                    let arg_str = self.translate_expr(arg);
                    if let Expression::Identifier(aname) = &arg.data { self.mark_moved(aname); }
                    arg_strs.push(arg_str);
                }
                format!("{}({})", name, arg_strs.join(", "))
            }
            Expression::MethodCall { receiver, method, args } => {
                let (obj_ty, is_ptr) = self.get_type_info(receiver);
                let receiver_raw = self.translate_receiver(receiver);
                if method == "clone" {
                    if is_ptr { return format!("{}_clone({})", obj_ty, receiver_raw); }
                    else { return format!("{}_clone(&{})", obj_ty, receiver_raw); }
                }
                let mut arg_strs = Vec::new();
                if is_ptr { arg_strs.push(receiver_raw); }
                else { arg_strs.push(format!("&{}", receiver_raw)); }
                for arg in args {
                    let arg_str = self.translate_expr(arg);
                    if let Expression::Identifier(aname) = &arg.data { self.mark_moved(aname); }
                    arg_strs.push(arg_str);
                }
                format!("{}_{}({})", obj_ty, method, arg_strs.join(", "))
            }
            Expression::MemberAccess { receiver, member, is_ptr: force_ptr } => {
                let (_, is_ptr) = self.get_type_info(receiver);
                let receiver_raw = self.translate_receiver(receiver);
                let op = if is_ptr || *force_ptr { "->" } else { "." };
                format!("{}{}{}", receiver_raw, op, member)
            }
        }
    }

    fn get_type_info(&self, expr_node: &Node<Expression>) -> (String, bool) {
        match &expr_node.data {
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
        for _ in 0..self.indent_level { self.output.push_str("    "); }
    }
}
