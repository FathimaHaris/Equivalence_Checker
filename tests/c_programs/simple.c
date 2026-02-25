// Simple function - no KLEE headers needed!
int compute(int x, int y) {
    if (x > 10) {
        return x + y;
    } else {
        return x * y;
    }
}