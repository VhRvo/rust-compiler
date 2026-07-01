use adder::ast::Expression;
use adder::parser::ProgParser;
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
        Ok(expr) => {
            // println!(";; Here's the original program: {}", input);
            println!(";; Here's the parsed abstract syntax tree: {:?}", expr);
            println!(
                ";; The result of the interpreter with input 483 is {}",
                interpret(&expr, 483)
            );
            println!(";; Result of the compiler:");
            optimized_compile(&expr);
        }
    }
}

fn interpret(e: &Expression, x: i64) -> i64 {
    match e {
        Expression::Variable() => x,
        Expression::Number(n) => *n,
        Expression::Add1(arg) => interpret(arg, x) + 1,
        Expression::Sub1(arg) => interpret(arg, x) - 1,
    }
}

// compiles the expression, printing the generated code directly to stdout
fn compile(e: &Expression) {
    fn compile_rec(e: &Expression) {
        match e {
            Expression::Variable() => println!("mov rax, rdi"),
            Expression::Number(n) => println!("mov rax, {}", n),
            Expression::Add1(arg) => {
                compile_rec(arg);
                println!("add rax, 1");
            }
            Expression::Sub1(arg) => {
                compile_rec(arg);
                println!("sub rax, 1");
            }
        }
    }
    println!(
        "        section .text
        global start_here
    start_here:"
    );
    compile_rec(e);
    println!("ret");
}

enum Optimized {
    Constant,
    InputPlus,
}

fn optimized_compile(e: &Expression) {
    fn ocompile_help(e: &Expression) -> (Optimized, i64) {
        match e {
            Expression::Variable() => (Optimized::InputPlus, 0),
            Expression::Number(n) => (Optimized::Constant, *n),
            Expression::Add1(arg) => {
                let (flag, n) = ocompile_help(arg);
                (flag, n + 1)
            }
            Expression::Sub1(arg) => {
                let (flag, n) = ocompile_help(arg);
                (flag, n - 1)
            }
        }
    }
    println!(
        "        section .text
        global start_here
    start_here:"
    );
    let (flag, n) = ocompile_help(e);
    match flag {
        Optimized::Constant => {
            println!("mov rax, {}", n);
        }
        Optimized::InputPlus => {
            println!("mov rax, rdi");
            if n != 0 {
                println!("add rax, {}", n);
            }
        }
    }
    println!("ret");
}
