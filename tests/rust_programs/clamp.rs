// tests/rust_programs/clamp_bug.rs
// BUG: conditions are flipped — lo/hi swapped
fn clamp(x: i32, lo: i32, hi: i32) -> i32 {
    if x > hi { return hi; }   // BUG: should be x < lo → return lo
    if x < lo { return lo; }   // BUG: should be x > hi → return hi
    x
}