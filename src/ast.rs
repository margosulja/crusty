#[derive(Debug, Clone)]
pub enum Expr {
    Identifier(String),
    Number(f64),
    String(String),
    BinaryOp {
        left: Box<Expr>,
        op: Binop,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Binop {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
pub enum Stmt {
    Expression(Expr),
    VariableDecl(VariableDecl), /* name, value */
    FunctionDecl(FunctionDecl),
}

#[derive(Debug)]
pub struct VariableDecl {
    pub data_type: String,
    pub name: String,
    pub value: Expr,
}

#[derive(Debug)]
pub struct FunctionDecl {
    pub data_type: String,
    pub name: String,
    pub body: Vec<Stmt>,
}

impl Binop {
    /*
        A precedence of operators system to ensure proper parsing.
    */
    pub fn precedence(&self) -> u8 {
        match self {
            Binop::Add | Binop::Sub => 1,
            Binop::Mul | Binop::Div => 2,
        }
    }

    /*
        All operators are linked with something to the left, for example:
        2 + 5, the left link is 2.
    */
    pub fn is_left_linked(&self) -> bool {
        true
    }
}
