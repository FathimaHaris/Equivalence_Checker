#include <klee/klee.h>

int main() {
  int x;
  klee_make_symbolic(&x, sizeof(x), "x");

  if (x == 7) {
    klee_report_error(__FILE__, __LINE__, "x == 7 reached", "klee_smoke");
  }

  return 0;
}
