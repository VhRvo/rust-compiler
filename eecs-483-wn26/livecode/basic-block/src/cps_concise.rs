use crate::{
    ast,
    cps_concise::DestinationPolicy::{Preferred, Required},
    ssa,
};
use std::cell::Cell;

enum DestinationPolicy {
    Required(String),
    Preferred(String),
}

impl DestinationPolicy {
    fn into_name(self) -> String {
        match self {
            DestinationPolicy::Preferred(name)
            | DestinationPolicy::Required(name) => name,
        }
    }
}

type Continuation<'a> = Box<dyn FnOnce(ssa::Immediate) -> ssa::BlockBody + 'a>;

pub fn lower(program: &ast::Program) -> ssa::Program {
    let next_name = Cell::new(0);
    let destination = Preferred("%result".to_string());
    let done = Box::new(|imm| ssa::BlockBody::Return(imm));

    ssa::Program {
        param: program.parameter.clone(),
        entry: lower_exp(&program.body, destination, done, &next_name),
    }
}

fn continue_with_immediate(
    destination: DestinationPolicy,
    immediate: ssa::Immediate,
    continuation: Continuation,
) -> ssa::BlockBody {
    match destination {
        DestinationPolicy::Preferred(_) => continuation(immediate),
        DestinationPolicy::Required(dest) => ssa::BlockBody::Operation {
            dest: dest.clone(),
            op: ssa::Operation::Immediate(immediate),
            next: Box::new(continuation(ssa::Immediate::Var(dest))),
        },
    }
}

fn continue_with_operation(
    destination: DestinationPolicy,
    operation: ssa::Operation,
    continuation: Continuation,
) -> ssa::BlockBody {
    let dest = destination.into_name();
    ssa::BlockBody::Operation {
        dest: dest.clone(),
        op: operation,
        next: Box::new(continuation(ssa::Immediate::Var(dest))),
    }
}

fn lower_exp(
    expression: &ast::Expression,
    destination: DestinationPolicy,
    continuation: Continuation,
    next_name: &Cell<usize>,
) -> ssa::BlockBody {
    match expression {
        ast::Expression::Variable(name) => {
            continue_with_immediate(destination, ssa::Immediate::Var(name.clone()), continuation)
        }
        ast::Expression::Number(number) => {
            continue_with_immediate(destination, ssa::Immediate::Const(*number), continuation)
        }
        ast::Expression::Let(name, rhs, body) => lower_exp(
            rhs,
            Required(name.clone()),
            Box::new(move |_imm| {
                assert_eq!(ssa::Immediate::Var(name.clone()), _imm);
                lower_exp(body, destination, continuation, next_name)
            }),
            next_name,
        ),
        ast::Expression::Prim { prim, args } => match (prim, args.as_slice()) {
            (ast::Prim::Add1, [argument]) => lower_unary(
                argument,
                ssa::Prim::Add,
                destination,
                continuation,
                next_name,
            ),
            (ast::Prim::Sub1, [argument]) => lower_unary(
                argument,
                ssa::Prim::Sub,
                destination,
                continuation,
                next_name,
            ),
            (ast::Prim::Add, [left, right]) => lower_binary(
                left,
                right,
                ssa::Prim::Add,
                destination,
                continuation,
                next_name,
            ),
            (ast::Prim::Sub, [left, right]) => lower_binary(
                left,
                right,
                ssa::Prim::Sub,
                destination,
                continuation,
                next_name,
            ),
            (ast::Prim::Mul, [left, right]) => lower_binary(
                left,
                right,
                ssa::Prim::Mul,
                destination,
                continuation,
                next_name,
            ),
            _ => panic!("invalid number of arguments for primitive {prim:?}"),
        },
    }
}

fn lower_unary<'a>(
    argument: &'a ast::Expression,
    primitive: ssa::Prim,
    destination: DestinationPolicy,
    continuation: Continuation,
    next_name: &'a Cell<usize>,
) -> ssa::BlockBody {
    lower_exp(
        argument,
        Preferred(fresh_name("%u", next_name)),
        Box::new(move |argument| {
            let operation = ssa::Operation::Prim(primitive, argument, ssa::Immediate::Const(1));
            continue_with_operation(destination, operation, continuation)
        }),
        next_name,
    )
}

fn lower_binary<'a>(
    left: &'a ast::Expression,
    right: &'a ast::Expression,
    primitive: ssa::Prim,
    destination: DestinationPolicy,
    continuation: Continuation,
    next_name: &'a Cell<usize>,
) -> ssa::BlockBody {
    // Nested continuations preserve left-to-right evaluation order.
    lower_exp(
        left,
        Preferred(fresh_name("%lhs", next_name)),
        Box::new(move |left| {
            lower_exp(
                right,
                Preferred(fresh_name("%rhs", next_name)),
                Box::new(move |right| {
                    let operation = ssa::Operation::Prim(primitive, left, right);
                    continue_with_operation(destination, operation, continuation)
                }),
                next_name,
            )
        }),
        next_name,
    )
}

fn fresh_name(prefix: &str, next_name: &Cell<usize>) -> String {
    let id = next_name.get();
    next_name.set(id + 1);
    format!("{}_{}", prefix, id)
}
