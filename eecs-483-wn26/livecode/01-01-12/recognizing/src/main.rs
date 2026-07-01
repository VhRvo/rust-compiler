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
        Ok(ast) => println!("the program looks good to me: {}", ast),
        Err(e) => println!("parse error: {}", e),
    }
}
