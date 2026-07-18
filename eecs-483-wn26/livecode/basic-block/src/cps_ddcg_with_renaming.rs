//! 完整组合版本：同时使用 CPS、destination-driven code generation 和
//! alpha-renaming。continuation 负责组织后续代码，DestinationPolicy
//! 避免不必要的 immediate 复制，环境则保证变量遮蔽后的引用仍指向
//! 正确且唯一的 SSA 定义。

use crate::{
    ast,
    cps_ddcg_with_renaming::DestinationPolicy::{Preferred, Required},
    ssa,
};
use im::HashMap;
use std::cell::Cell;

enum DestinationPolicy {
    // 必须产生指定 SSA 定义，例如 let 的绑定。
    Required(String),
    // immediate 可以直接复用；只有 operation 才写入该候选 destination。
    Preferred(String),
}

impl DestinationPolicy {
    // operation 必须拥有 destination，因此两种策略在此都转化为具体名字。
    fn into_name(self) -> String {
        match self {
            DestinationPolicy::Preferred(name) | DestinationPolicy::Required(name) => name,
        }
    }
}

type Continuation<'a> = Box<dyn FnOnce(ssa::Immediate) -> ssa::BlockBody + 'a>;

pub fn lower(program: &ast::Program) -> ssa::Program {
    // 一个 lowering 过程只使用一个计数器，所以不同前缀也不会产生同名 SSA 值。
    let next_name = Cell::new(0);
    // 顶层复合表达式直接写入 %result；顶层 immediate 则直接返回。
    let destination = Preferred("%result".to_string());
    let done = Box::new(|imm| ssa::BlockBody::Return(imm));
    // 参数已经是现成定义，初始环境将源码参数名映射到自身。
    let env = HashMap::unit(program.parameter.clone(), program.parameter.clone());

    ssa::Program {
        param: program.parameter.clone(),
        entry: lower_exp(env, &program.body, destination, done, &next_name),
    }
}

fn continue_with_immediate(
    destination: DestinationPolicy,
    immediate: ssa::Immediate,
    continuation: Continuation,
) -> ssa::BlockBody {
    // Required 保留显式定义；Preferred 对已有 immediate 消除复制。
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
    // operation 必须先写入 destination，之后才能以 Var immediate 继续传递。
    let dest = destination.into_name();
    ssa::BlockBody::Operation {
        dest: dest.clone(),
        op: operation,
        next: Box::new(continuation(ssa::Immediate::Var(dest))),
    }
}

fn lower_exp(
    env: HashMap<String, String>,
    expression: &ast::Expression,
    destination: DestinationPolicy,
    continuation: Continuation,
    next_name: &Cell<usize>,
) -> ssa::BlockBody {
    match expression {
        ast::Expression::Variable(name) => {
            // 查找当前词法作用域中该源码名对应的 SSA 名。
            let renamed = env.get(name).cloned().unwrap_or_else(|| name.clone());
            continue_with_immediate(destination, ssa::Immediate::Var(renamed), continuation)
        }
        ast::Expression::Number(number) => {
            continue_with_immediate(destination, ssa::Immediate::Const(*number), continuation)
        }
        ast::Expression::Let(name, rhs, body) => {
            // 每个 let 都获得唯一 SSA 名；% 前缀使内部名字与源码名字分区，
            // 全局递增编号则保证不同 let 和编译器临时变量之间也不会重名。
            let renamed = fresh_name(name, next_name);

            // let 不是递归绑定：RHS 在旧环境中 lowering，body 才能看到新绑定。
            let original_env = env.clone();
            let extended_env = env.update(name.clone(), renamed.clone());
            lower_exp(
                original_env,
                rhs,
                Required(renamed.clone()),
                Box::new(move |_imm| {
                    // Required 的协议保证 RHS 最终定义并返回 renamed。
                    assert_eq!(ssa::Immediate::Var(renamed), _imm);
                    lower_exp(extended_env, body, destination, continuation, next_name)
                }),
                next_name,
            )
        }
        ast::Expression::Prim { prim, args } => match (prim, args.as_slice()) {
            (ast::Prim::Add1, [argument]) => lower_unary(
                env,
                argument,
                ssa::Prim::Add,
                destination,
                continuation,
                next_name,
            ),
            (ast::Prim::Sub1, [argument]) => lower_unary(
                env,
                argument,
                ssa::Prim::Sub,
                destination,
                continuation,
                next_name,
            ),
            (ast::Prim::Add, [left, right]) => lower_binary(
                env,
                left,
                right,
                ssa::Prim::Add,
                destination,
                continuation,
                next_name,
            ),
            (ast::Prim::Sub, [left, right]) => lower_binary(
                env,
                left,
                right,
                ssa::Prim::Sub,
                destination,
                continuation,
                next_name,
            ),
            (ast::Prim::Mul, [left, right]) => lower_binary(
                env,
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
    env: HashMap<String, String>,
    argument: &'a ast::Expression,
    primitive: ssa::Prim,
    destination: DestinationPolicy,
    continuation: Continuation,
    next_name: &'a Cell<usize>,
) -> ssa::BlockBody {
    // immediate 参数直接复用；复合参数才会使用候选临时 destination。
    lower_exp(
        env,
        argument,
        Preferred(fresh_name("u", next_name)),
        Box::new(move |argument| {
            let operation = ssa::Operation::Prim(primitive, argument, ssa::Immediate::Const(1));
            continue_with_operation(destination, operation, continuation)
        }),
        next_name,
    )
}

fn lower_binary<'a>(
    env: HashMap<String, String>,
    left: &'a ast::Expression,
    right: &'a ast::Expression,
    primitive: ssa::Prim,
    destination: DestinationPolicy,
    continuation: Continuation,
    next_name: &'a Cell<usize>,
) -> ssa::BlockBody {
    // 左右操作数都从同一个外层环境开始；persistent environment 确保
    // 左侧内部 let 不会泄漏到右侧。嵌套 continuation 保持从左到右顺序。
    lower_exp(
        env.clone(),
        left,
        Preferred(fresh_name("lhs", next_name)),
        Box::new(move |left| {
            lower_exp(
                env,
                right,
                Preferred(fresh_name("rhs", next_name)),
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
    // 源码标识符不能包含 %，因此单个 % 已足以隔离用户命名空间；
    // 共享的递增编号进一步保证所有内部名字全局唯一。
    let id = next_name.get();
    next_name.set(id + 1);
    format!("%{}_{}", prefix, id)
}
