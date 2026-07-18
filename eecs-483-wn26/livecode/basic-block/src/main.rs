use im::HashMap;
use snake::ast;
use snake::parser::ProgParser;
use snake::ssa;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: {} input_file", args[0]);
        std::process::exit(1);
    }
    let input = fs::read_to_string(&args[1]).unwrap();
    let prog = ProgParser::new().parse(&input).unwrap();
    println!(
        "Here is the produced intermediate representation:\n{}",
        lower(&prog)
    );
}

fn interpret(p: &ast::Program, x: i64) -> i64 {
    fn interp_exp(e: &ast::Expression, mut env: HashMap<String, i64>) -> i64 {
        match e {
            ast::Expression::Variable(x) => *env.get(x).unwrap(),
            ast::Expression::Let(x, e1, e2) => {
                let v = interp_exp(e1, env.clone());
                env.insert(x.to_string(), v.clone());
                interp_exp(e2, env)
            }
            ast::Expression::Number(n) => *n,
            ast::Expression::Prim { prim, args } => match prim {
                ast::Prim::Add => {
                    let res1 = interp_exp(&args[0], env.clone());
                    let res2 = interp_exp(&args[1], env);
                    res1 + res2
                }
                ast::Prim::Add1 => interp_exp(&args[0], env) + 1,
                ast::Prim::Sub1 => interp_exp(&args[0], env) - 1,
                ast::Prim::Sub => {
                    let res1 = interp_exp(&args[0], env.clone());
                    let res2 = interp_exp(&args[1], env);
                    res1 - res2
                }
                ast::Prim::Mul => {
                    let res1 = interp_exp(&args[0], env.clone());
                    let res2 = interp_exp(&args[1], env);
                    res1 * res2
                }
            },
        }
    }
    let env: HashMap<String, i64> = HashMap::unit(p.parameter.clone(), x);
    interp_exp(&p.body, env)
}

fn lower(p: &ast::Program) -> ssa::Program {
    // "Top-level continuation" simply returns the result
    let dest = format!("result");
    let body = ssa::BlockBody::Return(ssa::Immediate::Var(dest.clone()));
    ssa::Program {
        param: p.parameter.to_string(),
        entry: lower_exp(&p.body, (dest, body)),
    }
}

// A Continuation consists of
// 1. A destination variable to store the result of the current expression
// 2. An SSA BlockBody to be executed *after* the result is stored
type Continuation = (String, ssa::BlockBody);

// given an expression e and a continuation (dest, body), compile to a
// block that assigns the value e produces to dest and then executes
// body.
fn lower_exp(e: &ast::Expression, k: Continuation) -> ssa::BlockBody {
    match e {
        ast::Expression::Variable(x) => {
            let (dest, body) = k;
            ssa::BlockBody::Operation {
                dest: dest,
                op: ssa::Operation::Immediate(ssa::Immediate::Var(x.to_string())),
                next: Box::new(body),
            }
        }
        ast::Expression::Number(n) => {
            let (dest, body) = k;
            ssa::BlockBody::Operation {
                dest: dest,
                op: ssa::Operation::Immediate(ssa::Immediate::Const(*n)),
                next: Box::new(body),
            }
        }
        ast::Expression::Prim { prim, args } => match prim {
            ast::Prim::Add => {
                let arg1 = &args[0];
                let arg2 = &args[1];
                // TODO: generate *unique* variable names for these!
                let tmp1 = format!("addArg1");
                let tmp2 = format!("addArg2");
                let (dest, body) = k;
                lower_exp(
                    arg1,
                    (
                        tmp1.clone(),
                        lower_exp(
                            arg2,
                            (
                                tmp2.clone(),
                                ssa::BlockBody::Operation {
                                    op: ssa::Operation::Prim(
                                        ssa::Prim::Add,
                                        ssa::Immediate::Var(tmp1),
                                        ssa::Immediate::Var(tmp2),
                                    ),
                                    dest,
                                    next: Box::new(body),
                                },
                            ),
                        ),
                    ),
                )
            }
            _ => todo!(),
        },
        _ => todo!(),
    }
}
