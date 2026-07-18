#[link(name = "compiled_code", kind = "static")]

extern "sysv64" {
    #[link_name = "\x01gcd"]
    fn gcd(m: u64, n: u64) -> u64;
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: {} m n", args[0]);
        std::process::exit(0);
    }
    let m: u64 = args[1].parse().unwrap();
    let n: u64 = args[2].parse().unwrap();
    let output = unsafe { gcd(m, n) };
    println!(
        "The greatest common divisor of {} and {} is {}",
        m, n, output
    );
}
