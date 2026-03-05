pub fn count_even(n: i32) -> i32 {
    let mut count = 0;
    for i in 0..=n {
        if i % 2 == 0 {
            count += 1;
        }
    }
    count
}