use adder::ast::Expression;
use adder::parser::ExprParser;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: {} input_file", args[0]);
        std::process::exit(1);
    }
    let input = fs::read_to_string(&args[1]).unwrap();

    match ExprParser::new().parse(&input) {
        Err(e) => {
            eprintln!("parse error: {}", e);
            std::process::exit(1)
        }
        Ok(expr) => {
            println!(";; Here's the original program: {}", input);
            println!(";; Here's the parsed abstract syntax tree: {:?}", expr);
            println!(";; The result of the interpreter is {}", interpret(&expr));
            println!(";; Result of the compiler:");
            optimized_compile(&expr);
        }
    }
}

fn interpret(e: &Expression) -> i64 {
    match e {
        // n : &i64
        Expression::Number(n) => *n,
        Expression::Add1(e) => interpret(e) + 1,
        Expression::Sub1(e) => interpret(e) - 1,
    }
}

// compiles the expression, printing the generated code directly to stdout
fn compile(e: &Expression) {
    // Compile e to x86 code that places interpret(e) in rax
    fn compiler_helper(e: &Expression) {
        match e {
            // n : &i64
            Expression::Number(n) => println!("mov rax, {}", n),
            Expression::Add1(e) => {
                compiler_helper(e);
                println!("add rax, 1");
            }
            Expression::Sub1(e) => {
                compiler_helper(e);
                println!("sub rax, 1");
            }
        }
    }
    println!(
        "        section .text
        global start_here
start_here:"
    );
    compiler_helper(e);
    println!("ret");
}

fn optimized_compile(e: &Expression) {
    println!(
        "        section .text
        global start_here
start_here:
    mov rax, {}
    ret",
        interpret(e)
    )
}
