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
    Expression(Expression),
    Return(Option<Expression>),
    Block(Vec<Node<Statement>>),
    If {
        condition: Expression,
        then_branch: Box<Node<Statement>>,
        else_branch: Option<Box<Node<Statement>>>,
    },
    RawC(String),
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
    pub default_value: Option<Expression>,
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
    Remove(String),
    Patch {
        old_fn_name: String,
        new_fn_def: FunctionDef,
    },
}

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub ty: String,
    pub name: String,
    pub init: Option<Expression>,
    pub constructor_call: Option<ConstructorCall>,
}

#[derive(Debug, Clone)]
pub struct ConstructorCall {
    pub name: String,
    pub args: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Identifier(String),
    Number(String),
    BinaryOp(Box<Expression>, String, Box<Expression>),
    MethodCall {
        receiver: Box<Expression>,
        method: String,
        args: Vec<Expression>,
    },
    MemberAccess {
        receiver: Box<Expression>,
        member: String,
        is_ptr: bool,
    },
    Assignment(String, Box<Expression>),
    StringLiteral(String),
    Call(String, Vec<Expression>),
}
