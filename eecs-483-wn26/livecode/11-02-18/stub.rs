#[repr(C)]
#[derive(PartialEq, Eq, Copy, Clone)]
struct SNAKE_VAL(u64);

static INT_TAG: u64 = 0x00_00_00_00_00_00_00_01;
static SNAKE_TRU: SNAKE_VAL = SNAKE_VAL(0b101);
static SNAKE_FLS: SNAKE_VAL = SNAKE_VAL(0b001);

#[link(name = "compiled_code", kind = "static")]
extern "sysv64" {
    #[link_name = "\x01entry"]
    fn entry(x: SNAKE_VAL, y: SNAKE_VAL) -> SNAKE_VAL;
}

// reinterprets the bytes of an unsigned number as a signed number
fn unsigned_to_signed(x: u64) -> i64 {
    i64::from_le_bytes(x.to_le_bytes())
}

fn signed_to_unsigned(x: i64) -> u64 {
    u64::from_le_bytes(x.to_le_bytes())
}

fn sprint_snake_val(x: SNAKE_VAL) -> String {
    if x.0 & INT_TAG == 0 {
        // it's a signed 63-bit integer
        format!("{}", unsigned_to_signed(x.0) >> 1)
    } else if x == SNAKE_TRU {
        String::from("true")
    } else if x == SNAKE_FLS {
        String::from("false")
    } else {
        String::from("INVALID_SNAKE_VALUE")
    }
}

fn parse_snake_val(s: &str) -> SNAKE_VAL {
    match s {
        "true" => SNAKE_TRU,
        "false" => SNAKE_FLS,
        _ => match s.parse::<i64>() {
            Ok(n) => SNAKE_VAL(signed_to_unsigned(n << 1)),
            Err(_) => panic!("Expected argument to be a snake value but got {}", s),
        },
    }
}

type ErrorCode = u64;
static ARITH_ERROR: ErrorCode = 0;
static COMPARE_ERROR: ErrorCode = 1;
static LOGIC_ERROR: ErrorCode = 2;

#[export_name = "\x01snake_error"]
extern "sysv64" fn snake_error(err_code: u64) {
    if err_code == ARITH_ERROR {
        eprintln!("arithmetic operation expected integer input(s)")
    } else if err_code == COMPARE_ERROR {
        eprintln!("comparison operation expected integer input(s)")
    } else if err_code == LOGIC_ERROR {
        eprintln!("logical operation expected boolean input(s)")
    } else {
        eprintln!("BUG IN COMPILER: invalid error code {}", err_code)
    }
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: {} snake_val snake_val", args[0]);
        std::process::exit(0);
    }
    let x = parse_snake_val(&args[1]);
    let y = parse_snake_val(&args[2]);
    let output = unsafe { entry(x, y) };
    println!("As unsigned int: {}", output.0);
    println!(
        "As   signed int: {}",
        i64::from_le_bytes(output.0.to_le_bytes())
    );
    println!("As binary:       0b{:064b}", output.0);
    println!("As hex:          0x{:016x}", output.0);
    println!("As snake value:  {}", sprint_snake_val(output));
}
