#include <stdio.h>

int unused_global = 10;

int unused_function() {
    return 5;
}

int f(int x) {
    int a = x + 1;
    int b = x + 2;   // used
    int c = x + 3;   // unused (dead instruction)

    return b;
}

int main() {
    printf("%d\n", f(5));
    return 0;
}