//! 带 alpha-renaming 的 CPS lowering。与基础版本相比，本版本为每个 let
//! 绑定分配唯一 SSA 名，并通过环境保存“源码名 -> 当前 SSA 名”的映射，
//! 因而能够正确处理词法作用域和变量遮蔽。

use crate::{ast, ssa};
use im::HashMap;
use std::cell::Cell;

// persistent map 允许不同词法作用域廉价地共享未改变的映射。
type Environment = HashMap<String, String>;

// continuation 描述“拿到当前表达式的 immediate 结果后做什么”。
// 闭包可能借用输入 AST 的子树，因此其生命周期不一定是 'static。
type Continuation<'a> = Box<dyn FnOnce(ssa::Immediate) -> ssa::BlockBody + 'a>;

pub fn lower(program: &ast::Program) -> ssa::Program {
    let next_name = Cell::new(0);
    // 参数本身已经是一个可用的 SSA 定义，初始环境将它映射到自身。
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
            // 使用当前词法作用域中的最新 SSA 名。fallback 交由其他阶段处理
            // 未绑定变量；合法的已解析程序通常都能在环境中找到名字。
            let renamed = environment
                .get(name)
                .cloned()
                .unwrap_or_else(|| name.clone());
            continuation(ssa::Immediate::Var(renamed))
        }
        ast::Expression::Number(number) => continuation(ssa::Immediate::Const(*number)),
        ast::Expression::Let(name, value, body) => {
            // let 不是递归绑定：RHS 必须在扩展环境之前 lowering。
            let value_environment = environment.clone();
            lower_exp(
                value,
                value_environment,
                next_name,
                Box::new(move |value| {
                    // 每个 SSA destination 必须唯一。body 使用加入新绑定后的
                    // 环境，因此内层同名 let 会遮蔽外层映射，但不会修改外层环境。
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
    // 左右操作数处于同一个外层词法环境；左侧内部的 let 不应泄漏到右侧。
    let right_environment = environment.clone();

    // continuation 的嵌套固定了从左到右的顺序：
    // 先 lowering 左操作数，再 lowering 右操作数，最后生成 primitive。
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
    // primitive 的结果先写入唯一临时变量，再作为 immediate 传给 continuation。
    let destination = fresh_name(next_name);
    let rest = continuation(ssa::Immediate::Var(destination.clone()));

    ssa::BlockBody::Operation {
        dest: destination,
        op: ssa::Operation::Prim(primitive, left, right),
        next: Box::new(rest),
    }
}

fn fresh_name(next_name: &Cell<usize>) -> String {
    // 源码标识符不能以下划线开头，因此该命名空间不会和解析得到的源码名冲突。
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
