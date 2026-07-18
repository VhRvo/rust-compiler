// Adderish + Print
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Exp {
    Num(i64),
    Var(String),
    Print(Box<Exp>),
    Let {
        bindings: Vec<(String, Exp)>, // new binding declarations
        body: Box<Exp>,               // the expression in which the new variables are bound
    },
    Add(Box<Exp>, Box<Exp>),
}
