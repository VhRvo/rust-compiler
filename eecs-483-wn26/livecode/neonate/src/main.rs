use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: snake <input.neonate>")
    }
    let input = fs::read_to_string(&args[1]).unwrap();
    let num: i64 = input.trim().parse().unwrap();

    println!(
        "        section .text
        global start_here
start_here:
        mov rax, {}
        ret",
        num
    );
}
