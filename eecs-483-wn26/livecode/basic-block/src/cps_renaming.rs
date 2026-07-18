use crate::{ast, ssa};
use im::HashMap;
use std::cell::Cell;

type Environment = HashMap<String, String>;

// A continuation says what to do with the immediate value produced by an
// expression. The lifetime is important: these closures may borrow subtrees
// from the input AST, so they are not necessarily 'static.
type Continuation<'a> = Box<dyn FnOnce(ssa::Immediate) -> ssa::BlockBody + 'a>;

pub fn lower(program: &ast::Program) -> ssa::Program {
    let next_name = Cell::new(0);
    let environment = Environment::unit(program.parameter.clone(), program.parameter.clone());

    ssa::Program {
        param: program.parameter.clone(),
        entry: lower_exp(
            &program.body,
            environment,
            &next_name,
            Box::new(ssa::BlockBody::Return),
        ),
    }
}

fn lower_exp<'a>(
    expression: &'a ast::Expression,
    environment: Environment,
    next_name: &'a Cell<usize>,
    continuation: Continuation<'a>,
) -> ssa::BlockBody {
    match expression {
        ast::Expression::Variable(name) => {
            let renamed = environment
                .get(name)
                .cloned()
                .unwrap_or_else(|| name.clone());
            continuation(ssa::Immediate::Var(renamed))
        }
        ast::Expression::Number(number) => continuation(ssa::Immediate::Const(*number)),
        ast::Expression::Let(name, value, body) => {
            let value_environment = environment.clone();
            lower_exp(
                value,
                value_environment,
                next_name,
                Box::new(move |value| {
                    // Every SSA destination must be unique. Keep a map from
                    // source names to their current SSA names so shadowing
                    // remains correct.
                    let destination = fresh_name(next_name);
                    let body_environment = environment.update(name.clone(), destination.clone());
                    let rest = lower_exp(body, body_environment, next_name, continuation);

                    ssa::BlockBody::Operation {
                        dest: destination,
                        op: ssa::Operation::Immediate(value),
                        next: Box::new(rest),
                    }
                }),
            )
        }
        ast::Expression::Prim { prim, args } => match (prim, args.as_slice()) {
            (ast::Prim::Add1, [arg]) => {
                lower_unary(arg, environment, next_name, ssa::Prim::Add, continuation)
            }
            (ast::Prim::Sub1, [arg]) => {
                lower_unary(arg, environment, next_name, ssa::Prim::Sub, continuation)
            }
            (ast::Prim::Add, [left, right]) => lower_binary(
                left,
                right,
                environment,
                next_name,
                ssa::Prim::Add,
                continuation,
            ),
            (ast::Prim::Sub, [left, right]) => lower_binary(
                left,
                right,
                environment,
                next_name,
                ssa::Prim::Sub,
                continuation,
            ),
            (ast::Prim::Mul, [left, right]) => lower_binary(
                left,
                right,
                environment,
                next_name,
                ssa::Prim::Mul,
                continuation,
            ),
            _ => panic!("invalid number of arguments for primitive {prim:?}"),
        },
    }
}

fn lower_unary<'a>(
    argument: &'a ast::Expression,
    environment: Environment,
    next_name: &'a Cell<usize>,
    primitive: ssa::Prim,
    continuation: Continuation<'a>,
) -> ssa::BlockBody {
    lower_exp(
        argument,
        environment,
        next_name,
        Box::new(move |argument| {
            emit_primitive(
                primitive,
                argument,
                ssa::Immediate::Const(1),
                next_name,
                continuation,
            )
        }),
    )
}

fn lower_binary<'a>(
    left: &'a ast::Expression,
    right: &'a ast::Expression,
    environment: Environment,
    next_name: &'a Cell<usize>,
    primitive: ssa::Prim,
    continuation: Continuation<'a>,
) -> ssa::BlockBody {
    let right_environment = environment.clone();

    // The nesting of the continuations fixes left-to-right evaluation order:
    // lower left, then lower right, then emit the primitive operation.
    lower_exp(
        left,
        environment,
        next_name,
        Box::new(move |left| {
            lower_exp(
                right,
                right_environment,
                next_name,
                Box::new(move |right| {
                    emit_primitive(primitive, left, right, next_name, continuation)
                }),
            )
        }),
    )
}

fn emit_primitive(
    primitive: ssa::Prim,
    left: ssa::Immediate,
    right: ssa::Immediate,
    next_name: &Cell<usize>,
    continuation: Continuation<'_>,
) -> ssa::BlockBody {
    let destination = fresh_name(next_name);
    let rest = continuation(ssa::Immediate::Var(destination.clone()));

    ssa::BlockBody::Operation {
        dest: destination,
        op: ssa::Operation::Prim(primitive, left, right),
        next: Box::new(rest),
    }
}

fn fresh_name(next_name: &Cell<usize>) -> String {
    let id = next_name.get();
    next_name.set(id + 1);
    format!("_cps_{id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowers_nested_arithmetic() {
        let program = ast::Program {
            parameter: "input".to_string(),
            body: ast::Expression::Prim {
                prim: ast::Prim::Add,
                args: vec![
                    ast::Expression::Number(1),
                    ast::Expression::Prim {
                        prim: ast::Prim::Mul,
                        args: vec![
                            ast::Expression::Number(2),
                            ast::Expression::Variable("input".to_string()),
                        ],
                    },
                ],
            },
        };

        let lowered = lower(&program);
        assert_eq!(
            lowered.entry,
            ssa::BlockBody::Operation {
                dest: "_cps_0".to_string(),
                op: ssa::Operation::Prim(
                    ssa::Prim::Mul,
                    ssa::Immediate::Const(2),
                    ssa::Immediate::Var("input".to_string()),
                ),
                next: Box::new(ssa::BlockBody::Operation {
                    dest: "_cps_1".to_string(),
                    op: ssa::Operation::Prim(
                        ssa::Prim::Add,
                        ssa::Immediate::Const(1),
                        ssa::Immediate::Var("_cps_0".to_string()),
                    ),
                    next: Box::new(ssa::BlockBody::Return(ssa::Immediate::Var(
                        "_cps_1".to_string(),
                    ))),
                }),
            }
        );
    }

    #[test]
    fn renames_shadowed_let_variables() {
        let program = ast::Program {
            parameter: "input".to_string(),
            body: ast::Expression::Let(
                "x".to_string(),
                Box::new(ast::Expression::Number(1)),
                Box::new(ast::Expression::Let(
                    "x".to_string(),
                    Box::new(ast::Expression::Number(2)),
                    Box::new(ast::Expression::Variable("x".to_string())),
                )),
            ),
        };

        assert_eq!(
            lower(&program).entry,
            ssa::BlockBody::Operation {
                dest: "_cps_0".to_string(),
                op: ssa::Operation::Immediate(ssa::Immediate::Const(1)),
                next: Box::new(ssa::BlockBody::Operation {
                    dest: "_cps_1".to_string(),
                    op: ssa::Operation::Immediate(ssa::Immediate::Const(2)),
                    next: Box::new(ssa::BlockBody::Return(ssa::Immediate::Var(
                        "_cps_1".to_string(),
                    ))),
                }),
            }
        );
    }
}
