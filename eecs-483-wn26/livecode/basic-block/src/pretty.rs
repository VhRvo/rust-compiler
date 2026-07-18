use std::fmt::{self, Display};

/* ------------------------------- Combinators ------------------------------ */

struct Indent<'a, T: ?Sized>(usize, &'a T);

/* ----------------------------- Implementations ---------------------------- */
mod impl_ssa {
    use super::*;
    use crate::ssa::*;

    impl Display for Program {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "entry({}):\n{}", self.param, Indent(1, &self.entry))
        }
    }

    impl Display for Indent<'_, BlockBody> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", "  ".repeat(self.0))?;
            match self.1 {
                BlockBody::Return(imm) => write!(f, "ret {}", imm),
                BlockBody::Operation { dest, op, next } => {
                    write!(f, "{} = {}\n{}", dest, op, Indent(self.0, next.as_ref()))
                }
            }
        }
    }

    impl Display for Operation {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Operation::Immediate(imm) => write!(f, "{}", imm),
                Operation::Prim(prim, imm1, imm2) => write!(f, "{} {} {}", prim, imm1, imm2),
            }
        }
    }

    impl Display for Prim {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Prim::Add => write!(f, "add"),
                Prim::Sub => write!(f, "sub"),
                Prim::Mul => write!(f, "mul"),
            }
        }
    }

    impl Display for Immediate {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Immediate::Const(c) => write!(f, "{}", c),
                Immediate::Var(v) => write!(f, "{}", v),
            }
        }
    }
}
