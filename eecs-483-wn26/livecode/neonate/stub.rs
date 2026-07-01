#[link(name = "compiled_code", kind = "static")]
extern "sysv64" {
    #[link_name = "\u{1}start_here"]
    fn start_here() -> i64;
}

fn main() {
    let output = unsafe { start_here() };
    println!("Assembly code returned: {}", output);
}
