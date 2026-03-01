fn in_range(x: i32) -> i32 {
    if x > 0 && x <= 10 {  // Bug: should be >=, not >
        return 1;
    } else {
        return 0;
    }
}