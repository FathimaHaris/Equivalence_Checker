int in_range(int x) {
    if (x < 0) {
        return -1;  // Different value for negative
    } else if (x <= 10) {
        return 1;   // In range
    } else {
        return x;   // Different value for >10
    }
}