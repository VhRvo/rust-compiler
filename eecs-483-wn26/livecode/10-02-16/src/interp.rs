use crate::ast::*;

// Cobra semantics: coercions
fn bool_to_int(b: bool) -> i64 {
    if b {
        1
    } else {
        0
    }
}

fn int_to_bool(n: i64) -> bool {
    n != 0
}
pub fn interp_boa(e: &Expression) -> i64 {
    match e {
        Expression::Integer(n) => *n,
        Expression::Boolean(b) => {
            if *b {
                1
            } else {
                0
            }
        }
        Expression::Prim1(Prim1::IsInt, _e) => bool_to_int(true),
        Expression::Prim1(Prim1::IsBool, _e) => bool_to_int(true),
        Expression::If(cond, thn, els) => {
            if int_to_bool(inter_boa(cond)) {
                interp_boa(thn)
            } else {
                interp_boa(els)
            }
        }
        Expression::Prim1(Prim1::Not, e) => bool_to_int(!(int_to_bool(interp_boa(e)))),
        Expression::Prim2(Prim2::Add, e1, e2) => interp_boa(e1) + interp_boa(e2),
        Expression::Prim2(Prim2::Sub, e1, e2) => interp_boa(e1) - interp_boa(e2),
        Expression::Prim2(Prim2::Mul, e1, e2) => interp_boa(e1) * interp_boa(e2),
        Expression::Prim2(Prim2::LEq, e1, e2) => bool_to_int(interp_boa(e1) <= interp_boa(e2)),
        Expression::Prim2(Prim2::LT, e1, e2) => bool_to_int(interp_boa(e1) < interp_boa(e2)),
        Expression::Prim2(Prim2::GEq, e1, e2) => bool_to_int(interp_boa(e1) >= interp_boa(e2)),
        Expression::Prim2(Prim2::GT, e1, e2) => bool_to_int(interp_boa(e1) > interp_boa(e2)),
        Expression::Prim2(Prim2::And, e1, e2) => {
            bool_to_int(int_to_bool(interp_boa(e1)) && int_to_bool(interp_boa(e2)))
        }
        Expression::Prim2(Prim2::Or, e1, e2) => {
            bool_to_int(int_to_bool(interp_boa(e1)) || int_to_bool(interp_boa(e2)))
        }
    }
}

// Diamondback semantics: dynamic typing

pub enum RuntimeError {
    ArithExpectedInt,
    CompareExpectedInt,
    LogExpectedBool,
    IfExpectedBool,
}

#[derive(Copy, Clone)]
pub enum SnakeVal {
    Integer(i64),
    Boolean(bool),
}

fn assert_int(v: SnakeVal) -> Option<i64> {
    match v {
        SnakeVal::Integer(n) => Some(n),
        SnakeVal::Boolean(_) => None,
    }
}

fn assert_bool(v: SnakeVal) -> Option<bool> {
    match v {
        SnakeVal::Boolean(b) => Some(b),
        SnakeVal::Integer(_) => None,
    }
}

pub fn interp_arith(
    e1: &Expression,
    e2: &Expression,
    op: impl Fn(i64, i64) -> i64,
) -> Result<SnakeVal, RuntimeError> {
    let v1 = interp_diamond(e1)?;
    let v2 = interp_diamond(e2)?;
    let n1 = assert_int(v1).ok_or(RuntimeError::ArithExpectedInt)?;
    let n2 = assert_int(v2).ok_or(RuntimeError::ArithExpectedInt)?;
    Ok(SnakeVal::Integer(op(n1, n2)))
}

pub fn interp_cmp(
    e1: &Expression,
    e2: &Expression,
    op: impl Fn(i64, i64) -> bool,
) -> Result<SnakeVal, RuntimeError> {
    let v1 = interp_diamond(e1)?;
    let v2 = interp_diamond(e2)?;
    let n1 = assert_int(v1).ok_or(RuntimeError::CompareExpectedInt)?;
    let n2 = assert_int(v2).ok_or(RuntimeError::CompareExpectedInt)?;
    Ok(SnakeVal::Boolean(op(n1, n2)))
}

pub fn interp_log(
    e1: &Expression,
    e2: &Expression,
    op: impl Fn(bool, bool) -> bool,
) -> Result<SnakeVal, RuntimeError> {
    let v1 = interp_diamond(e1)?;
    let v2 = interp_diamond(e2)?;
    let b1 = assert_bool(v1).ok_or(RuntimeError::LogExpectedBool)?;
    let b2 = assert_bool(v2).ok_or(RuntimeError::LogExpectedBool)?;
    Ok(SnakeVal::Boolean(op(b1, b2)))
}

pub fn interp_diamond(e: &Expression) -> Result<SnakeVal, RuntimeError> {
    match e {
        Expression::Integer(n) => Ok(SnakeVal::Integer(*n)),

        Expression::Boolean(b) => Ok(SnakeVal::Boolean(*b)),

        Expression::Prim1(Prim1::IsInt, e) => match interp_diamond(e)? {
            SnakeVal::Integer(_) => Ok(SnakeVal::Boolean(true)),
            SnakeVal::Boolean(_) => Ok(SnakeVal::Boolean(false)),
        },

        Expression::Prim1(Prim1::IsBool, e) => match interp_diamond(e)? {
            SnakeVal::Integer(_) => Ok(SnakeVal::Boolean(false)),
            SnakeVal::Boolean(_) => Ok(SnakeVal::Boolean(true)),
        },

        Expression::If(cond, thn, els) => {
            let b: bool = assert_bool(interp_diamond(cond)?).ok_or(RuntimeError::IfExpectedBool)?;
            if b {
                interp_diamond(thn)
            } else {
                interp_diamond(els)
            }
        }

        Expression::Prim1(Prim1::Not, e) => {
            let v = interp_diamond(e)?;
            let b = assert_bool(v).ok_or(RuntimeError::LogExpectedBool)?;
            Ok(SnakeVal::Boolean(!b))
        }

        Expression::Prim2(Prim2::Add, e1, e2) => {
            let v1: SnakeVal = interp_diamond(e1)?;
            let v2: SnakeVal = interp_diamond(e2)?;
            let n1: i64 = assert_int(v1).ok_or(RuntimeError::ArithExpectedInt)?;
            let n2: i64 = assert_int(v2).ok_or(RuntimeError::ArithExpectedInt)?;
            Ok(SnakeVal::Integer(n1 + n2))
        }

        Expression::Prim2(Prim2::Sub, e1, e2) => interp_arith(e1, e2, |n1, n2| n1 - n2),
        Expression::Prim2(Prim2::Mul, e1, e2) => interp_arith(e1, e2, |n1, n2| n1 * n2),
        Expression::Prim2(Prim2::LEq, e1, e2) => interp_cmp(e1, e2, |n1, n2| n1 <= n2),
        Expression::Prim2(Prim2::LT, e1, e2) => interp_cmp(e1, e2, |n1, n2| n1 < n2),
        Expression::Prim2(Prim2::GEq, e1, e2) => interp_cmp(e1, e2, |n1, n2| n1 >= n2),
        Expression::Prim2(Prim2::GT, e1, e2) => interp_cmp(e1, e2, |n1, n2| n1 > n2),
        Expression::Prim2(Prim2::And, e1, e2) => interp_log(e1, e2, |b1, b2| b1 && b2),
        Expression::Prim2(Prim2::Or, e1, e2) => interp_log(e1, e2, |b1, b2| b1 || b2),
    }
}
