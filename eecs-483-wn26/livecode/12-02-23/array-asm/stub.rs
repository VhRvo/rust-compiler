#[repr(C)]
#[derive(PartialEq, Eq, Copy, Clone)]
struct SnakeVal(u64);

#[repr(C)]
struct SnakeArray {
    size: u64,
    elts: *const SnakeVal,
}

static HEAP_SIZE: u64 = 100000;
static mut HEAP_START: [u64; 100000] = [0; 100000];
static mut HEAP_PTR: *mut u64 = unsafe { HEAP_START.as_mut_ptr() };

// Casts a pointer to an array to a more convenient interface
fn load_snake_array(p: *const u64) -> SnakeArray {
    unsafe {
        let size = *p;
        SnakeArray {
            size,
            elts: std::mem::transmute(p.add(1)),
        }
    }
}

static INT_TAG: u64 = 0b01;
static FULL_TAG: u64 = 0b11;
static SNAKE_TRU: SnakeVal = SnakeVal(0b101);
static SNAKE_FLS: SnakeVal = SnakeVal(0b001);

#[link(name = "compiled_code", kind = "static")]
extern "sysv64" {
    #[link_name = "\x01entry"]
    fn entry(x: SnakeVal) -> SnakeVal;
}

// reinterprets the bytes of an unsigned number as a signed number
fn unsigned_to_signed(x: u64) -> i64 {
    i64::from_le_bytes(x.to_le_bytes())
}

fn signed_to_unsigned(x: i64) -> u64 {
    u64::from_le_bytes(x.to_le_bytes())
}

fn sprint_snake_val(x: SnakeVal) -> String {
    if x.0 & INT_TAG == 0 {
        // it's a signed 63-bit integer
        format!("{}", unsigned_to_signed(x.0) >> 1)
    } else if x.0 & FULL_TAG == FULL_TAG {
        // value is a tagged pointer
        let untagged = x.0 ^ FULL_TAG;
        let arr = load_snake_array(untagged as *const u64);
        format!("Array(addr=0x{:016x}, len={})", untagged, arr.size)
    } else if x == SNAKE_TRU {
        String::from("true")
    } else if x == SNAKE_FLS {
        String::from("false")
    } else {
        String::from("INVALID_SnakeValUE")
    }
}

fn parse_snake_val(s: &str) -> SnakeVal {
    match s {
        "true" => SNAKE_TRU,
        "false" => SNAKE_FLS,
        _ => match s.parse::<i64>() {
            Ok(n) => SnakeVal(signed_to_unsigned(n << 1)),
            Err(_) => panic!("Expected argument to be a snake value but got {}", s),
        },
    }
}

type ErrorCode = u64;
static ARITH_ERROR: ErrorCode = 0;
static COMPARE_ERROR: ErrorCode = 1;
static LOGIC_ERROR: ErrorCode = 2;

#[export_name = "\x01print_snake_value"]
extern "sysv64" fn print_snake_value(x: SnakeVal) -> SnakeVal {
    println!("{}", sprint_snake_val(x));
    SnakeVal(0)
}

#[export_name = "\x01new_array"]
extern "sysv64" fn new_array(size: u64) -> SnakeVal {
    let arr_ptr = unsafe { HEAP_PTR as u64 };
    unsafe {
        if arr_ptr + 8 * (size + 1) >= (HEAP_START.as_ptr() as u64) + 8 * HEAP_SIZE {
            eprintln!("Out of memory");
            std::process::exit(1);
        }
        *HEAP_PTR = size;
        for _i in 0..size {
            HEAP_PTR = HEAP_PTR.add(1);
            *HEAP_PTR = 0;
        }
        HEAP_PTR = HEAP_PTR.add(1)
    }
    SnakeVal(arr_ptr | FULL_TAG)
}

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
    unsafe {
        if HEAP_START.as_ptr() as u64 & FULL_TAG != 0 {
            eprintln!("the heap is misaligned!");
            std::process::exit(1);
        }
    }
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: {} snake_val", args[0]);
        std::process::exit(0);
    }
    let x = parse_snake_val(&args[1]);
    let output = unsafe { entry(x) };
    println!("As unsigned int: {}", output.0);
    println!(
        "As   signed int: {}",
        i64::from_le_bytes(output.0.to_le_bytes())
    );
    println!("As binary:       0b{:064b}", output.0);
    println!("As hex:          0x{:016x}", output.0);
    println!("As snake value:  {}", sprint_snake_val(output));
}
