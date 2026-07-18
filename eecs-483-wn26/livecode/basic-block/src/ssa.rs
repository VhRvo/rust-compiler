type VarName = String;

// A Program has a single input parameter, and a block of straightline code to execute
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub param: VarName,
    pub entry: BlockBody,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockBody {
    Return(Immediate),
    Operation {
        dest: VarName,
        op: Operation,
        next: Box<BlockBody>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    Immediate(Immediate),
    Prim(Prim, Immediate, Immediate),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Prim {
    // binary
    Add,
    Sub,
    Mul,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Immediate {
    Const(i64),
    Var(VarName),
}
