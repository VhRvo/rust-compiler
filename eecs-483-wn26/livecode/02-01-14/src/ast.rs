type Var = String;
/*
 *  def main(x):
 *    e
 *
 * */
#[derive(Clone, Debug)]
pub struct Program {
    pub parameter: Var,
    pub body: Expression,
}

#[derive(Clone, Debug)]
pub enum Expression {
    Variable(Var),
    Number(i64),
    Add1(Box<Expression>),
    Sub1(Box<Expression>),
    // let x = e1 in e2
    Let(Var, Box<Expression>, Box<Expression>),
}
