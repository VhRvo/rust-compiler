#[derive(Clone, Debug)]
pub enum Expression {
    Variable(),
    Number(i64),
    Add1(Box<Expression>),
    Sub1(Box<Expression>),
}
