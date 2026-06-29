type AST = i64;

fn main() {
    use std::fs;

    let args = std::env::args().collect::<Vec<String>>();
    let input = fs::read_to_string(&args[1]).unwrap();
    let num = parse(&input).unwrap();
    println!("{}", compile(num));
}

fn parse(input: &str) -> Result<AST, String> {
    match i64::from_str_radix(input.trim(), 10) {
        Ok(input) => Ok(input),
        Err(e) => Err(e.to_string()),
    }
}

fn compile(n: AST) -> String {
    format!(
        "\
    section .text
    global start_here
start_here:
    mov rax, {}
    ret",
        n
    )
}
