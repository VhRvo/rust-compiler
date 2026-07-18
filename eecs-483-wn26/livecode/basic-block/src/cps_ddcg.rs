//! destination-driven code generation 与 CPS 的组合版本。
//! destination 决定当前表达式应把结果写到哪里，continuation 决定得到
//! immediate 后如何继续。这个版本没有 alpha-renaming，因此和基础 CPS
//! 版本一样，假设源码中不存在变量遮蔽。

use crate::{
    ast,
    cps_ddcg::DestinationPolicy::{Preferred, Required},
    ssa,
};
use std::cell::Cell;

enum DestinationPolicy {
    // 必须把结果写入指定名字；let 绑定即使 RHS 是 immediate 也需要该赋值。
    Required(String),
    // 优先使用指定名字；若结果本来就是 immediate，则直接传给 continuation，
    // 只有真正生成 operation 时才使用该 destination。
    Preferred(String),
}

impl DestinationPolicy {
    // operation 必须拥有 destination，因此此时两种策略都可提取出名字。
    fn into_name(self) -> String {
        match self {
            DestinationPolicy::Preferred(name)
            | DestinationPolicy::Required(name) => name,
        }
    }
}

type Continuation<'a> = Box<dyn FnOnce(ssa::Immediate) -> ssa::BlockBody + 'a>;

pub fn lower(program: &ast::Program) -> ssa::Program {
    // 所有临时变量共享该计数器，以保证生成名唯一。
    let next_name = Cell::new(0);
    // 顶层若产生 operation，就直接写入 %result；若本来是 immediate，
    // Preferred 允许直接 ret，避免生成“%result = immediate”的多余复制。
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
    // immediate 不需要计算。Required 为 let 等上下文保留显式绑定；
    // Preferred 则绕过无意义的复制，直接把已有 immediate 向后传递。
    match destination {
        DestinationPolicy::Preferred(_) => continuation(immediate),
        DestinationPolicy::Required(dest) => ssa::BlockBody::Operation {
            // Required 的协议保证 continuation 收到的就是该 destination。
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
    // operation 一定要落到某个 SSA destination，然后才能作为变量被后续使用。
    // Required 和 Preferred 都可以提取出名字，交给 operation 使用。
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
        // let 的 RHS 必须绑定到源码指定的名字。这里没有重命名，故不支持遮蔽。
        ast::Expression::Let(name, rhs, body) => lower_exp(
            rhs,
            Required(name.clone()),
            Box::new(move |_imm| {
                // Required 的协议保证 continuation 收到的就是该 destination。
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
    // 参数若为 immediate，Preferred 会直接复用它；若为复合表达式，
    // 其 operation 才会写入预先准备的临时 destination。
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
    // 嵌套 continuation 保持从左到右的 lowering 顺序。左右参数的
    // Preferred destination 可能不会实际使用，因此编号允许出现空洞。
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
    // 所有调用共享计数器；% 前缀还使临时变量与源码标识符命名空间分离。
    let id = next_name.get();
    next_name.set(id + 1);
    format!("{}_{}", prefix, id)
}
