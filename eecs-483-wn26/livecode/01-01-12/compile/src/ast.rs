#[derive(Clone, Debug)]
pub enum Expression {
    Number(i64),
    Add1(Box<Expression>),
    Sub1(Box<Expression>),
}

// Expression::Number(5);
// Expression::Add1(Box::new(Expression::Number(5)));
// Expression::Sub1(Box::new(Expression::Add1(Box::new(Expression::Number(5)))));
