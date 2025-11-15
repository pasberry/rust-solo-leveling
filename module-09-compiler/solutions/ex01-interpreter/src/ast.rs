#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(i64),
    Boolean(bool),
    String(String),
    Identifier(String),
    Array(Vec<Expr>),
    Hash(Vec<(Expr, Expr)>),
    Index {
        left: Box<Expr>,
        index: Box<Expr>,
    },
    Prefix {
        operator: PrefixOp,
        right: Box<Expr>,
    },
    Infix {
        left: Box<Expr>,
        operator: InfixOp,
        right: Box<Expr>,
    },
    If {
        condition: Box<Expr>,
        consequence: Vec<Stmt>,
        alternative: Option<Vec<Stmt>>,
    },
    Function {
        parameters: Vec<String>,
        body: Vec<Stmt>,
    },
    Call {
        function: Box<Expr>,
        arguments: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let { name: String, value: Expr },
    Assign { name: String, value: Expr },
    Return(Expr),
    Expression(Expr),
    While { condition: Expr, body: Vec<Stmt> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrefixOp {
    Minus,
    Bang,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InfixOp {
    Plus,
    Minus,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    And,
    Or,
}
