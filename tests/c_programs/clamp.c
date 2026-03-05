int clamp(int x, int low, int high) {
    if (x < low) return low;
    if (x > high) return high;
    return x;
}