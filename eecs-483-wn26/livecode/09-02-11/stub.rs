#[link(name = "compiled_code", kind = "static")]
extern "sysv64" {
    #[link_name = "\x01entry"]
    fn entry(x: i64) -> i64;
}

#[export_name = "\x01read"]
extern "sysv64" fn read() -> i64 {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();
    buf.trim().parse().unwrap()
}

#[export_name = "\x01print"]
extern "sysv64" fn print(x: i64) {
    println!("{}", x);
}

#[export_name = "\x01big_fun_nine"]
extern "sysv64" fn big_fun_nine(
    x1: i64,
    x2: i64,
    x3: i64,
    x4: i64,
    x5: i64,
    x6: i64,
    x7: i64,
    x8: i64,
    x9: i64,
) -> i64 {
    println!(
        "x1: {}\nx2: {}\nx3: {}\nx4: {}\nx5: {}\nx6: {}\nx7: {}\nx8: {}\nx9: {}",
        x1, x2, x3, x4, x5, x6, x7, x8, x9
    );
    x1 + x2 + x3 + x4 + x5 + x6 + x7 + x8 + x9
}

#[export_name = "\x01big_fun_ten"]
extern "sysv64" fn big_fun_ten(
    x1: i64,
    x2: i64,
    x3: i64,
    x4: i64,
    x5: i64,
    x6: i64,
    x7: i64,
    x8: i64,
    x9: i64,
    x10: i64,
) -> i64 {
    println!(
        "x1: {}\nx2: {}\nx3: {}\nx4: {}\nx5: {}\nx6: {}\nx7: {}\nx8: {}\nx9: {}\nx10",
        x1, x2, x3, x4, x5, x6, x7, x8, x9
    );
    x1 + x2 + x3 + x4 + x5 + x6 + x7 + x8 + x9 + x10
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: {} number", args[0]);
        std::process::exit(0);
    }
    let x: i64 = args[1].parse().unwrap();
    let output = unsafe { entry(x) };
    println!("Assembly code returned: {}", output);
}
