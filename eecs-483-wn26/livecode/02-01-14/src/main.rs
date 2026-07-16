use im::{HashMap, HashSet};
use snake::ast::{Expression, Program};
use snake::parser::ProgParser;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: {} input_file", args[0]);
        std::process::exit(1);
    }
    let input = fs::read_to_string(&args[1]).unwrap();

    match ProgParser::new().parse(&input) {
        Err(e) => {
            eprintln!("parse error: {}", e);
            std::process::exit(1)
        }
        Ok(prog) => {
            println!(";; Here's the parsed abstract syntax tree: {:?}", prog);
            match check_scope(&prog) {
                Err(msg) => {
                    eprintln!("scope checking error: {}", msg);
                    std::process::exit(1);
                }
                Ok(()) => {
                    println!("scope checking succeeded");
                    println!(
                        ";; The result of the interpreter with input 483 is {}",
                        interpret(&prog, 483)
                    );
                    todo!("compile");
                }
            }
        }
    }
}

type CompileError = String;
// return Ok(()) if the program has no free variables
// return Err(e) where e reports an occurrence of a free variable
fn check_scope(p: &Program) -> Result<(), CompileError> {
    fn check_scope_exp<'a>(
        e: &'a Expression,
        mut env: HashSet<&'a str>,
    ) -> Result<(), CompileError> {
        match e {
            Expression::Variable(x) => {
                if env.contains(x.as_str()) {
                    Ok(())
                } else {
                    Err(format!("Free occurrence of the variable {}", x))
                }
            }
            // let x = e1 in e2
            Expression::Let(x, e1, e2) => {
                check_scope_exp(e1, env.clone())?;
                env.insert(x);
                check_scope_exp(e2, env)
            }
            Expression::Number(_) => Ok(()),
            Expression::Add1(e) => check_scope_exp(e, env),
            Expression::Sub1(e) => check_scope_exp(e, env),
        }
    }
    let env = HashSet::unit(p.parameter.as_str());
    check_scope_exp(&p.body, env)
}

fn interpret(p: &Program, x: i64) -> i64 {
    fn interp_exp<'a>(e: &'a Expression, mut env: HashMap<&'a str, i64>) -> i64 {
        match e {
            // x
            Expression::Variable(x) => *env.get(x.as_str()).unwrap(),
            // let x = e1 in e2
            Expression::Let(x, e1, e2) => {
                let v = interp_exp(e1, env.clone());
                env.insert(x, v);
                interp_exp(e2, env)
            }
            Expression::Number(n) => *n,
            Expression::Add1(e) => interp_exp(e, env) + 1,
            Expression::Sub1(e) => interp_exp(e, env) - 1,
        }
    }
    let env: HashMap<&str, i64> = HashMap::unit(p.parameter.as_str(), x);
    interp_exp(&p.body, env)
}

fn add1(x: i64) -> i64 {
    x + 1
}

fn sub1(x: i64) -> i64 {
    x - 1
}

fn funny(x: i64) -> i64 {
    let z = add1(add1({
        let y = sub1(x);
        y
    }));
    add1(z)
}
