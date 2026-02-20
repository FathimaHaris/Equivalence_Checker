// tests/rust_programs/simple.rs
pub fn compute(x: i32, y: i32) -> i32 {
    if x > 10 {
        return x + y;
    } else {
        return x * y;
    }
}