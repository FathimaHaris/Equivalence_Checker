fn max(x: i32, y: i32) -> i32 {
    if x > y {
        x
    } else {
        x  // Bug: returns x instead of y
    }
}