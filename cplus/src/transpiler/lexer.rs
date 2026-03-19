use crate::transpiler::ast::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenData {
    Keyword(String),
    Identifier(String),
    Number(String),
    Operator(String),
    Symbol(char),
    StringLiteral(String),
    RawC(String),
    Whitespace,
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub data: TokenData,
    pub span: Span,
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0, line: 1, col: 1 }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let span = Span { line: self.line, col: self.col };

        if self.pos >= self.input.len() {
            return Token { data: TokenData::EOF, span };
        }

        let ch = self.current_char();

        // Comments
        if ch == '/' && self.peek_char() == '/' {
            self.skip_comment();
            return self.next_token();
        }

        // Identifiers & Keywords
        if ch.is_alphabetic() || ch == '_' {
            let ident = self.read_identifier();
            let data = match ident.as_str() {
                "let" | "unsafe" | "struct" | "bind" | "fork" | "patch" | "host" | "as" | "alias" | "mut" | "return" | "if" | "else" | "spawn" | "for" | "while" => {
                    TokenData::Keyword(ident)
                }
                _ => TokenData::Identifier(ident),
            };
            return Token { data, span };
        }

        // Numbers
        if ch.is_numeric() {
            return Token { data: TokenData::Number(self.read_number()), span };
        }

        // Strings
        if ch == '"' {
            return Token { data: TokenData::StringLiteral(self.read_string()), span };
        }

        // Preprocessor or Raw C (starting with #)
        if ch == '#' {
            return Token { data: TokenData::RawC(self.read_raw_line()), span };
        }

        // Multi-char operators
        if ch == '-' && self.peek_char() == '>' {
            self.advance(); self.advance();
            return Token { data: TokenData::Operator("->".to_string()), span };
        }
        if (ch == '+' && self.peek_char() == '+') || 
           (ch == '-' && self.peek_char() == '-') ||
           (ch == '=' && self.peek_char() == '=') ||
           (ch == '!' && self.peek_char() == '=') ||
           (ch == '<' && self.peek_char() == '=') ||
           (ch == '>' && self.peek_char() == '=') ||
           (ch == '+' && self.peek_char() == '=') ||
           (ch == '-' && self.peek_char() == '=') {
            let mut op = ch.to_string();
            op.push(self.peek_char());
            self.advance(); self.advance();
            return Token { data: TokenData::Operator(op), span };
        }

        // Symbols and single-char operators
        let data = match ch {
            '{' | '}' | '(' | ')' | '[' | ']' | ';' | ',' | ':' => {
                self.advance();
                TokenData::Symbol(ch)
            }
            '+' | '-' | '*' | '/' | '=' | '.' | '&' | '<' | '>' | '!' => {
                self.advance();
                TokenData::Operator(ch.to_string())
            }
            _ => {
                self.advance();
                TokenData::Whitespace // Ignore unknown
            }
        };

        if data == TokenData::Whitespace {
            return self.next_token();
        }

        Token { data, span }
    }

    fn current_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or('\0')
    }

    fn peek_char(&self) -> char {
        self.input[self.pos + 1..].chars().next().unwrap_or('\0')
    }

    fn advance(&mut self) {
        if let Some(ch) = self.input[self.pos..].chars().next() {
            self.pos += ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.current_char().is_whitespace() {
            self.advance();
        }
    }

    fn skip_comment(&mut self) {
        while self.pos < self.input.len() && self.current_char() != '\n' {
            self.advance();
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut res = String::new();
        while self.pos < self.input.len() && (self.current_char().is_alphanumeric() || self.current_char() == '_') {
            res.push(self.current_char());
            self.advance();
        }
        res
    }

    fn read_number(&mut self) -> String {
        let mut res = String::new();
        while self.pos < self.input.len() && (self.current_char().is_numeric() || self.current_char() == '.') {
            res.push(self.current_char());
            self.advance();
        }
        res
    }

    fn read_string(&mut self) -> String {
        self.advance(); // skip "
        let mut res = String::new();
        while self.pos < self.input.len() && self.current_char() != '"' {
            res.push(self.current_char());
            self.advance();
        }
        self.advance(); // skip "
        res
    }

    fn read_raw_line(&mut self) -> String {
        let mut res = String::new();
        while self.pos < self.input.len() && self.current_char() != '\n' {
            res.push(self.current_char());
            self.advance();
        }
        if self.current_char() == '\n' {
            self.advance();
        }
        res
    }
}
