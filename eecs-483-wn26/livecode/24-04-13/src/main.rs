mod parser;
mod parser2; // with add, right associative
mod parser3; // with add, left associative
mod syntax;

use crate::parser::ExpParser;

fn main() {
    use std::io;
    let lines = io::stdin().lines();
    for line in lines {
        // match parser::ExpParser::new().parse(&line.unwrap()) {
        // match parser2::ExpParser::new().parse(&line.unwrap()) {
        match parser3::ExpParser::new().parse(&line.unwrap()) {
            Ok(exp) => println!("Successful parse: {:?}", exp),
            Err(err) => println!("Parse error: {:?}", err),
        }
    }
}
