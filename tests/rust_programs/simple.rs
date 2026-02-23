// tests/rust_programs/simple.rs
pub fn compute(x: i32, y: i32) -> i32 {
    if x > 10 {
        return x + y;
    } else {
        return x * y;
    }
}

// For KLEE, we need a C-compatible main
#[no_mangle]
pub extern "C" fn main() -> i32 {
    // KLEE will make these symbolic
    let x: i32 = 0;
    let y: i32 = 0;
    
    // Call the function
    compute(x, y)
}