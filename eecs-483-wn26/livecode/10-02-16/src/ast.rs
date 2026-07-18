#[derive(Clone, Debug)]
pub enum Expression {
    Integer(i64),
    Boolean(bool),
    Prim1(Prim1, Box<Expression>),
    Prim2(Prim2, Box<Expression>, Box<Expression>),
    If(Box<Expression>, Box<Expression>, Box<Expression>),
}

#[derive(Clone, Debug)]
pub enum Prim1 {
    Not,
    IsBool,
    IsInt,
}

#[derive(Clone, Debug)]
pub enum Prim2 {
    // Arithmetic
    Add,
    Sub,
    Mul,

    // Comparison
    LEq,
    LT,
    GEq,
    GT,

    // Logical
    And,
    Or,
}
