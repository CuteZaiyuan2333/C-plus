use crate::transpiler::lexer::{Lexer, Token, TokenData};
use crate::transpiler::ast::*;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    peek_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();
        println!("DEBUG: Initial tokens: {:?}, {:?}", current_token.data, peek_token.data);
        Self { lexer, current_token, peek_token }
    }

    fn advance(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn expect_symbol(&mut self, s: char) {
        match self.current_token.data {
            TokenData::Symbol(ch) if ch == s => self.advance(),
            _ => panic!("Parser Error at {}:{}: Expected symbol '{}', found {:?}", 
                       self.current_token.span.line, self.current_token.span.col, s, self.current_token.data),
        }
    }

    fn expect_operator(&mut self, op: &str) {
        match &self.current_token.data {
            TokenData::Operator(s) if s == op => self.advance(),
            _ => panic!("Parser Error at {}:{}: Expected operator '{}', found {:?}", 
                       self.current_token.span.line, self.current_token.span.col, op, self.current_token.data),
        }
    }

    pub fn parse(&mut self) -> Vec<Node<Statement>> {
        let mut stmts = Vec::new();
        while self.current_token.data != TokenData::EOF {
            if let Some(stmt) = self.parse_statement() {
                stmts.push(stmt);
            } else {
                self.advance();
            }
        }
        stmts
    }

    fn parse_statement(&mut self) -> Option<Node<Statement>> {
        let span = self.current_token.span;
        // println!("DEBUG: Parsing statement at {:?}, current token: {:?}", span, self.current_token.data);
        match &self.current_token.data {
            TokenData::Keyword(kw) => match kw.as_str() {
                "struct" => Some(Node { data: Statement::StructDef(self.parse_struct()), span }),
                "bind" => Some(Node { data: Statement::BindBlock(self.parse_bind()), span }),
                "fork" => Some(Node { data: Statement::ForkBlock(self.parse_fork()), span }),
                "let" => Some(self.parse_declaration(true)),
                "unsafe" => Some(self.parse_declaration(false)),
                "alias" => Some(Node { data: Statement::AliasDeclaration(self.parse_alias()), span }),
                "return" => Some(Node { data: Statement::Return(self.parse_return()), span }),
                "if" => Some(self.parse_if()),
                "spawn" => Some(self.parse_spawn()),
                _ => {
                    // println!("DEBUG: Keyword fell through to default: {:?}", kw);
                    let expr = self.parse_expression(0);
                    self.consume_optional_semi();
                    Some(Node { data: Statement::Expression(expr), span })
                }
            },
            TokenData::RawC(c) => {
                let content = c.clone();
                self.advance();
                Some(Node { data: Statement::RawC(content), span })
            },
            TokenData::Symbol('{') => Some(self.parse_block_node()),
            TokenData::Symbol(';') => { self.advance(); None },
            TokenData::Identifier(_) => {
                if let TokenData::Identifier(_) = &self.peek_token.data {
                    let ty_span = self.current_token.span;
                    let ty = self.read_identifier();
                    let name_span = self.current_token.span;
                    let name = self.read_identifier();
                    if let TokenData::Symbol('(') = self.current_token.data {
                        self.advance(); // (
                        let params = self.parse_parameters();
                        self.expect_symbol(')');
                        let body = self.parse_block_node();
                        Some(Node { data: Statement::FunctionDef(FunctionDef { name, params, return_ty: ty, body: Box::new(body) }), span })
                    } else {
                         let left = Node { data: Expression::Identifier(ty), span: ty_span };
                         let right = Node { data: Expression::Identifier(name), span: name_span };
                         let expr = Node { data: Expression::BinaryOp(Box::new(left), " ".to_string(), Box::new(right)), span };
                         Some(Node { data: Statement::Expression(expr), span })
                    }
                } else if let TokenData::Symbol('(') = &self.peek_token.data {
                    Some(Node { data: Statement::Expression(self.parse_expression(0)), span })
                } else {
                    let expr = self.parse_expression(0);
                    self.consume_optional_semi();
                    Some(Node { data: Statement::Expression(expr), span })
                }
            }
            _ => {
                let expr = self.parse_expression(0);
                self.consume_optional_semi();
                Some(Node { data: Statement::Expression(expr), span })
            }
        }
    }

    fn consume_optional_semi(&mut self) {
        if let TokenData::Symbol(';') = self.current_token.data {
            self.advance();
        }
    }

    fn parse_struct(&mut self) -> StructDef {
        self.advance(); // struct
        let name = self.read_identifier();
        self.expect_symbol('{');
        let mut fields = Vec::new();
        while !matches!(self.current_token.data, TokenData::Symbol('}') | TokenData::EOF) {
            let mut ty = self.read_identifier();
            // 支持指针类型，如 char*
            while let TokenData::Operator(ref op) = self.current_token.data {
                if op == "*" {
                    ty.push('*');
                    self.advance();
                } else {
                    break;
                }
            }
            let field_name = self.read_identifier();
            let mut default_value = None;
            if let TokenData::Operator(ref op) = self.current_token.data {
                if op == "=" {
                    self.advance();
                    default_value = Some(self.parse_expression(0));
                }
            }
            self.expect_symbol(';');
            fields.push(Field { name: field_name, ty, default_value });
        }
        self.expect_symbol('}');
        StructDef { name, fields }
    }

    fn parse_bind(&mut self) -> BindBlock {
        self.advance(); // bind
        let struct_name = self.read_identifier();
        self.expect_symbol('{');
        let mut functions = Vec::new();
        while !matches!(self.current_token.data, TokenData::Symbol('}') | TokenData::EOF) {
            functions.push(self.parse_function_inner());
        }
        self.expect_symbol('}');
        BindBlock { struct_name, functions }
    }

    fn parse_function_inner(&mut self) -> FunctionDef {
        let first = self.read_identifier();
        let mut name = first.clone();
        let mut return_ty = "void".to_string();
        
        match self.current_token.data {
            TokenData::Symbol('(') => {
                // Constructor: Name(...)
            }
            _ => {
                return_ty = first;
                name = self.read_identifier();
            }
        }
        
        self.expect_symbol('(');
        let params = self.parse_parameters();
        self.expect_symbol(')');
        let body = self.parse_block_node();
        FunctionDef { name, params, return_ty, body: Box::new(body) }
    }

    fn parse_parameters(&mut self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        while !matches!(self.current_token.data, TokenData::Symbol(')') | TokenData::EOF) {
            if let TokenData::Keyword(kw) = &self.current_token.data {
                if kw == "let" { self.advance(); }
            }
            let mut ty = self.read_identifier();
            while let TokenData::Operator(ref op) = self.current_token.data {
                if op == "*" {
                    ty.push('*');
                    self.advance();
                } else {
                    break;
                }
            }
            match &self.current_token.data {
                TokenData::Identifier(second) => {
                    let second_name = second.clone();
                    self.advance();
                    params.push((ty, second_name));
                }
                _ => {
                    params.push(("int".to_string(), ty));
                }
            }
            if let TokenData::Symbol(',') = self.current_token.data {
                self.advance();
            }
        }
        params
    }

    fn parse_block_node(&mut self) -> Node<Statement> {
        let span = self.current_token.span;
        self.expect_symbol('{');
        let mut stmts = Vec::new();
        while !matches!(self.current_token.data, TokenData::Symbol('}') | TokenData::EOF) {
            if let Some(s) = self.parse_statement() {
                stmts.push(s);
            }
        }
        self.expect_symbol('}');
        Node { data: Statement::Block(stmts), span }
    }

    fn parse_declaration(&mut self, is_let: bool) -> Node<Statement> {
        let span = self.current_token.span;
        self.advance(); // let/unsafe
        let mut ty = self.read_identifier();
        while let TokenData::Operator(ref op) = self.current_token.data {
            if op == "*" { ty.push('*'); self.advance(); } else { break; }
        }
        let name = self.read_identifier();
        let mut init = None;
        let mut constructor_call = None;

        match &self.current_token.data {
            TokenData::Operator(op) if op == "." => {
                self.advance();
                let c_name = self.read_identifier();
                self.expect_symbol('(');
                let args = self.parse_expression_list();
                self.expect_symbol(')');
                constructor_call = Some(ConstructorCall { _name: c_name, args });
            }
            TokenData::Operator(op) if op == "=" => {
                self.advance();
                init = Some(self.parse_expression(0));
            }
            _ => {}
        }
        self.expect_symbol(';');
        let decl = VariableDeclaration { ty, name, init, constructor_call };
        if is_let {
            Node { data: Statement::LetDeclaration(decl), span }
        } else {
            Node { data: Statement::UnsafeDeclaration(decl), span }
        }
    }

    fn parse_expression_list(&mut self) -> Vec<Node<Expression>> {
        let mut list = Vec::new();
        while !matches!(self.current_token.data, TokenData::Symbol(')') | TokenData::EOF) {
            list.push(self.parse_expression(0));
            if let TokenData::Symbol(',') = self.current_token.data {
                self.advance();
            }
        }
        list
    }

    fn parse_alias(&mut self) -> AliasDeclaration {
        self.advance(); // alias
        let mut is_mut = false;
        if let TokenData::Keyword(kw) = &self.current_token.data {
            if kw == "mut" { is_mut = true; self.advance(); }
        }
        let ty = self.read_identifier();
        let name = self.read_identifier();
        self.expect_operator("=");
        let source = self.read_identifier();
        self.expect_symbol(';');
        AliasDeclaration { is_mut, ty, name, source }
    }

    fn parse_fork(&mut self) -> ForkBlock {
        self.advance(); // fork
        let base_name = self.read_identifier();
        self.advance(); // as
        let new_name = self.read_identifier();
        self.expect_symbol('{');
        let mut operations = Vec::new();
        while !matches!(self.current_token.data, TokenData::Symbol('}') | TokenData::EOF) {
            match &self.current_token.data {
                TokenData::Operator(op) if op == "+" => {
                    self.advance();
                    let ty = self.read_identifier();
                    let name = self.read_identifier();
                    let mut default_value = None;
                    if let TokenData::Operator(ref op2) = self.current_token.data {
                        if op2 == "=" { self.advance(); default_value = Some(self.parse_expression(0)); }
                    }
                    self.expect_symbol(';');
                    operations.push(ForkOp::Add(Field { name, ty, default_value }));
                }
                TokenData::Operator(op) if op == "-" => {
                    self.advance();
                    let name = self.read_identifier();
                    self.expect_symbol(';');
                    operations.push(ForkOp::Remove(name));
                }
                _ => self.advance(),
            }
        }
        self.expect_symbol('}');
        
        let mut bind_ops = Vec::new();
        if let TokenData::Keyword(kw) = &self.current_token.data {
            if kw == "bind" {
                self.advance();
                if let TokenData::Identifier(_) = self.current_token.data { self.advance(); }
                self.expect_symbol('{');
                while !matches!(self.current_token.data, TokenData::Symbol('}') | TokenData::EOF) {
                    match &self.current_token.data {
                        TokenData::Operator(op) if op == "+" => {
                            self.advance();
                            bind_ops.push(BindOp::Add(self.parse_function_inner()));
                        }
                        TokenData::Operator(op) if op == "-" => {
                            self.advance();
                            let name = self.read_identifier();
                            if let TokenData::Symbol('(') = self.current_token.data {
                                self.advance(); self.expect_symbol(')');
                            }
                            self.expect_symbol(';');
                            bind_ops.push(BindOp::Remove(name));
                        }
                        TokenData::Keyword(kw) if kw == "patch" => {
                            self.advance();
                            let _old_name = self.read_identifier();
                            // Skip old signature up to 'as' or the start of new signature
                            while !matches!(self.current_token.data, TokenData::Keyword(ref k) if k == "as") && self.current_token.data != TokenData::EOF {
                                self.advance();
                            }
                            if let TokenData::Keyword(kw2) = &self.current_token.data {
                                if kw2 == "as" { self.advance(); }
                            }
                            bind_ops.push(BindOp::Patch { _old_fn_name: _old_name, new_fn_def: self.parse_function_inner() });
                        }
                        _ => bind_ops.push(BindOp::Add(self.parse_function_inner())),
                    }
                }
                self.expect_symbol('}');
            }
        }
        ForkBlock { base_name, new_name, operations, bind_ops }
    }

    fn parse_if(&mut self) -> Node<Statement> {
        let span = self.current_token.span;
        self.advance(); // if
        self.expect_symbol('(');
        let condition = self.parse_expression(0);
        self.expect_symbol(')');
        let then_branch = Box::new(self.parse_statement().expect("If must have a then branch"));
        let mut else_branch = None;
        if let TokenData::Keyword(kw) = &self.current_token.data {
            if kw == "else" {
                self.advance();
                else_branch = Some(Box::new(self.parse_statement().expect("Else must have a branch")));
            }
        }
        Node { data: Statement::If { condition, then_branch, else_branch }, span }
    }

    fn parse_return(&mut self) -> Option<Node<Expression>> {
        self.advance(); // return
        if let TokenData::Symbol(';') = self.current_token.data {
            self.advance();
            None
        } else {
            let expr = Some(self.parse_expression(0));
            self.expect_symbol(';');
            expr
        }
    }

    fn parse_spawn(&mut self) -> Node<Statement> {
        let span = self.current_token.span;
        self.advance(); // spawn
        let func_name = self.read_identifier();
        self.expect_symbol('(');
        let args = self.parse_expression_list();
        self.expect_symbol(')');
        self.consume_optional_semi();
        Node { data: Statement::Spawn(func_name, args), span }
    }

    fn parse_expression(&mut self, precedence: i8) -> Node<Expression> {
        let mut left = self.parse_primary();
        
        while let TokenData::Operator(ref op) = self.current_token.data {
            let p = self.get_precedence(op);
            if p <= precedence { break; }
            let op_clone = op.clone();
            self.advance();
            
            if op_clone == "." || op_clone == "->" {
                let member = self.read_identifier();
                if let TokenData::Symbol('(') = self.current_token.data {
                    self.advance();
                    let args = self.parse_expression_list();
                    self.expect_symbol(')');
                    let span = left.span;
                    left = Node { data: Expression::MethodCall { receiver: Box::new(left), method: member, args }, span };
                } else {
                    let span = left.span;
                    left = Node { data: Expression::MemberAccess { receiver: Box::new(left), member, is_ptr: op_clone == "->" }, span };
                }
            } else {
                let right = self.parse_expression(p);
                let span = left.span;
                left = Node { data: Expression::BinaryOp(Box::new(left), op_clone, Box::new(right)), span };
            }
        }
        left
    }

    fn parse_primary(&mut self) -> Node<Expression> {
        let span = self.current_token.span;
        match self.current_token.data.clone() {
            TokenData::Identifier(s) => {
                self.advance();
                if let TokenData::Symbol('(') = self.current_token.data {
                    self.advance();
                    let args = self.parse_expression_list();
                    self.expect_symbol(')');
                    Node { data: Expression::Call(s, args), span }
                } else {
                    Node { data: Expression::Identifier(s), span }
                }
            }
            TokenData::Keyword(s) if s == "host" => {
                self.advance();
                Node { data: Expression::Identifier("host".to_string()), span }
            }
            TokenData::Number(s) => { self.advance(); Node { data: Expression::Number(s), span } }
            TokenData::StringLiteral(s) => { self.advance(); Node { data: Expression::StringLiteral(s), span } }
            TokenData::Symbol('(') => {
                self.advance();
                let expr = self.parse_expression(0);
                self.expect_symbol(')');
                
                // C style cast: (type)expr
                // If the next token could start an expression, this might be a cast
                if let TokenData::Identifier(_) | TokenData::Number(_) | TokenData::Symbol('(') | TokenData::StringLiteral(_) = self.current_token.data {
                    if let Expression::Identifier(ty) = &expr.data {
                         let ty_clone = ty.clone();
                         let inner = self.parse_expression(100); // High precedence for cast
                         let cast_span = expr.span;
                         let left_node = Node { data: Expression::Identifier(format!("({})", ty_clone)), span: cast_span };
                         return Node { data: Expression::BinaryOp(Box::new(left_node), "".to_string(), Box::new(inner)), span: cast_span };
                    }
                }
                expr
            }
            _ => panic!("Parser Error at {}:{}: Unexpected token in expression: {:?}", 
                       self.current_token.span.line, self.current_token.span.col, self.current_token.data),
        }
    }

    fn get_precedence(&self, op: &str) -> i8 {
        match op {
            "=" => 1,
            "==" | "!=" | "<" | ">" => 2,
            "+" | "-" => 3,
            "*" | "/" => 4,
            "." | "->" => 5,
            _ => 0,
        }
    }

    fn read_identifier(&mut self) -> String {
        let res = match &self.current_token.data {
            TokenData::Identifier(s) => s.clone(),
            TokenData::Keyword(s) => {
                if s == "host" || s == "patch" || s == "as" || s == "mut" {
                    s.clone()
                } else {
                    panic!("Parser Error at {}:{}: Expected identifier, found keyword {:?}", 
                           self.current_token.span.line, self.current_token.span.col, s);
                }
            }
            _ => panic!("Parser Error at {}:{}: Expected identifier, found {:?}", 
                       self.current_token.span.line, self.current_token.span.col, self.current_token.data),
        };
        self.advance();
        res
    }
}
