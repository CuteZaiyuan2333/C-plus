#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct Node<T> {
    pub data: T,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Statement {
    StructDef(StructDef),
    BindBlock(BindBlock),
    ForkBlock(ForkBlock),
    FunctionDef(FunctionDef),
    LetDeclaration(VariableDeclaration),
    UnsafeDeclaration(VariableDeclaration),
    AliasDeclaration(AliasDeclaration),
    Expression(Node<Expression>),
    Return(Option<Node<Expression>>),
    Block(Vec<Node<Statement>>),
    If {
        condition: Node<Expression>,
        then_branch: Box<Node<Statement>>,
        else_branch: Option<Box<Node<Statement>>>,
    },
    For {
        init: Option<Box<Node<Statement>>>,
        condition: Option<Node<Expression>>,
        step: Option<Node<Expression>>,
        body: Box<Node<Statement>>,
    },
    While {
        condition: Node<Expression>,
        body: Box<Node<Statement>>,
    },
    RawC(String),
    Spawn(String, Vec<Node<Expression>>),
}

#[derive(Debug, Clone)]
pub struct AliasDeclaration {
    pub is_mut: bool,
    pub ty: String,
    pub name: String,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: String,
    pub default_value: Option<Node<Expression>>,
}

#[derive(Debug, Clone)]
pub struct BindBlock {
    pub struct_name: String,
    pub functions: Vec<FunctionDef>,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<(String, String)>,
    pub return_ty: String,
    pub body: Box<Node<Statement>>,
}

#[derive(Debug, Clone)]
pub struct ForkBlock {
    pub base_name: String,
    pub new_name: String,
    pub operations: Vec<ForkOp>,
    pub bind_ops: Vec<BindOp>,
}

#[derive(Debug, Clone)]
pub enum ForkOp {
    Add(Field),
    Remove(String),
}

#[derive(Debug, Clone)]
pub enum BindOp {
    Add(FunctionDef),
    #[allow(dead_code)]
    Remove(String),
    Patch {
        _old_fn_name: String,
        new_fn_def: FunctionDef,
    },
}

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub ty: String,
    pub name: String,
    pub init: Option<Node<Expression>>,
    pub constructor_call: Option<ConstructorCall>,
}

#[derive(Debug, Clone)]
pub struct ConstructorCall {
    pub _name: String,
    pub args: Vec<Node<Expression>>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Identifier(String),
    Number(String),
    BinaryOp(Box<Node<Expression>>, String, Box<Node<Expression>>),
    MethodCall {
        receiver: Box<Node<Expression>>,
        method: String,
        args: Vec<Node<Expression>>,
    },
    MemberAccess {
        receiver: Box<Node<Expression>>,
        member: String,
        is_ptr: bool,
    },
    #[allow(dead_code)]
    Assignment(String, Box<Node<Expression>>),
    UnaryOp(String, Box<Node<Expression>>, bool), // op, expr, is_postfix
    StringLiteral(String),
    Call(String, Vec<Node<Expression>>),
}
