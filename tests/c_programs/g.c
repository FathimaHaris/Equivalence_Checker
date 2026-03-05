int g(int *p, int *q, int n) {
  int s = 0;
  for (int i = 0; i < n; i++) {
    s += p[i];
    if (q) s += q[i];
  }
  return s;
}