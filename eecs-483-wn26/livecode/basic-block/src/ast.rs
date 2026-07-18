type Var = String;

#[derive(Clone, Debug)]
pub struct Program {
    pub parameter: Var,
    pub body: Expression,
}

#[derive(Clone, Debug)]
pub enum Expression {
    Variable(Var),
    Number(i64),
    Prim { prim: Prim, args: Vec<Expression> },
    Let(Var, Box<Expression>, Box<Expression>),
}

#[derive(Clone, Copy, Debug)]
pub enum Prim {
    // unary
    Add1,
    Sub1,
    // binary
    Add,
    Sub,
    Mul,
}
