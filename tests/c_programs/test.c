#include <stdio.h>

// int test(int x) {
//     int a = x * 2;      // should become shift
//     int b = a + 0;      // identity simplification
//     int c = b * 1;      // identity simplification
//     int d = 3 + 5;      // constant folding
//     int e = c - c;      // becomes 0

//     return a + b + c + d + e;
// }


int test(int x)
{
    x=5;
    int y=x+3;
    if (y>10)
    {
        return 1;
    }
    else
    {
        return 2;
    }
}

int main() {
    printf("%d\n", test(10));
    return 0;
}