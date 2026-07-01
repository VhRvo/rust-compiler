#[link(name = "compiled_code", kind = "static")]
extern "sysv64" {
    #[link_name = "\x01start_here"]
    fn start_here(x: i64) -> i64;
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: {} number", args[0]);
        std::process::exit(0);
    }
    let x: i64 = args[1].parse().unwrap();
    let output = unsafe { start_here(x) };
    println!("Assembly code returned: {}", output);
}
