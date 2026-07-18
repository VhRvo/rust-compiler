//! 基础 CPS lowering：continuation 负责接收子表达式产生的 immediate，
//! 再决定如何构造后续 SSA。这个版本保留源码中的 let 变量名，因此假设
//! 源程序不存在变量遮蔽；支持遮蔽的版本见 `cps_renaming`。

use crate::{ast, ssa};
use std::cell::Cell;

// continuation 描述“拿到当前表达式的 immediate 结果后做什么”。
// 闭包可能借用输入 AST 的子树，因此其生命周期不一定是 'static。
// 使用 FnOnce 是因为 lowering 过程中每条 continuation 只会被调用一次。
type Continuation<'a> = Box<dyn FnOnce(ssa::Immediate) -> ssa::BlockBody + 'a>;

pub fn lower(program: &ast::Program) -> ssa::Program {
    // 整个函数共享一个计数器，保证所有编译器临时变量互不重复。
    let next_name = Cell::new(0);

    ssa::Program {
        param: program.parameter.clone(),
        // 顶层 continuation 直接返回程序最终产生的 immediate。
        entry: lower_exp(&program.body, &next_name, Box::new(ssa::BlockBody::Return)),
    }
}

fn lower_exp<'a>(
    expression: &'a ast::Expression,
    next_name: &'a Cell<usize>,
    continuation: Continuation<'a>,
) -> ssa::BlockBody {
    match expression {
        ast::Expression::Variable(name) => continuation(ssa::Immediate::Var(name.clone())),
        ast::Expression::Number(number) => continuation(ssa::Immediate::Const(*number)),
        ast::Expression::Let(name, value, body) => {
            // 先 lowering RHS；RHS 的结果传给闭包后，再构造 let 绑定和 body。
            lower_exp(
                value,
                next_name,
                Box::new(move |value| {
                    // 简化处理：直接保留源码变量名，因此要求源程序没有变量遮蔽。
                    let rest = lower_exp(body, next_name, continuation);

                    ssa::BlockBody::Operation {
                        dest: name.clone(),
                        op: ssa::Operation::Immediate(value),
                        next: Box::new(rest),
                    }
                }),
            )
        }
        ast::Expression::Prim { prim, args } => match (prim, args.as_slice()) {
            (ast::Prim::Add1, [arg]) => lower_unary(arg, next_name, ssa::Prim::Add, continuation),
            (ast::Prim::Sub1, [arg]) => lower_unary(arg, next_name, ssa::Prim::Sub, continuation),
            (ast::Prim::Add, [left, right]) => {
                lower_binary(left, right, next_name, ssa::Prim::Add, continuation)
            }
            (ast::Prim::Sub, [left, right]) => {
                lower_binary(left, right, next_name, ssa::Prim::Sub, continuation)
            }
            (ast::Prim::Mul, [left, right]) => {
                lower_binary(left, right, next_name, ssa::Prim::Mul, continuation)
            }
            _ => panic!("invalid number of arguments for primitive {prim:?}"),
        },
    }
}

fn lower_unary<'a>(
    argument: &'a ast::Expression,
    next_name: &'a Cell<usize>,
    primitive: ssa::Prim,
    continuation: Continuation<'a>,
) -> ssa::BlockBody {
    // add1/sub1 在目标 SSA 中表示为与常量 1 的二元 add/sub。
    lower_exp(
        argument,
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
    next_name: &'a Cell<usize>,
    primitive: ssa::Prim,
    continuation: Continuation<'a>,
) -> ssa::BlockBody {
    // continuation 的嵌套固定了从左到右的顺序：
    // 先 lowering 左操作数，再 lowering 右操作数，最后生成 primitive。
    lower_exp(
        left,
        next_name,
        Box::new(move |left| {
            lower_exp(
                right,
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
    // primitive 的结果不是 immediate，必须先写入一个新的 SSA 临时变量，
    // 再把该变量作为 immediate 交给后续 continuation。
    let destination = fresh_name(next_name);
    let rest = continuation(ssa::Immediate::Var(destination.clone()));

    ssa::BlockBody::Operation {
        dest: destination,
        op: ssa::Operation::Prim(primitive, left, right),
        next: Box::new(rest),
    }
}

fn fresh_name(next_name: &Cell<usize>) -> String {
    // 所有调用共享 next_name，所以即使表达式任意嵌套也不会重名。
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
    fn lowers_let_binding() {
        let program = ast::Program {
            parameter: "input".to_string(),
            body: ast::Expression::Let(
                "x".to_string(),
                Box::new(ast::Expression::Number(1)),
                Box::new(ast::Expression::Variable("x".to_string())),
            ),
        };

        assert_eq!(
            lower(&program).entry,
            ssa::BlockBody::Operation {
                dest: "x".to_string(),
                op: ssa::Operation::Immediate(ssa::Immediate::Const(1)),
                next: Box::new(ssa::BlockBody::Return(
                    ssa::Immediate::Var("x".to_string(),)
                )),
            }
        );
    }
}
