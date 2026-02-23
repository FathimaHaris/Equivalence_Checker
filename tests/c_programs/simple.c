// tests/c_programs/simple.c
#include <klee/klee.h>

// The function we want to verify
int compute(int x, int y) {
    if (x > 10) {
        return x + y;
    } else {
        return x * y;
    }
}

// Main function for KLEE
int main() {
    int x, y;
    
    // Make inputs symbolic
    klee_make_symbolic(&x, sizeof(x), "x");
    klee_make_symbolic(&y, sizeof(y), "y");
    
    // Add bounds
    klee_assume(x >= 0 && x <= 100);
    klee_assume(y >= 0 && y <= 100);
    
    // Call the function
    int result = compute(x, y);
    
    return result;
}