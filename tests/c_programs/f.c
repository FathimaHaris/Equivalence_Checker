struct Point {
    int x;
    int y;
};

int f() {
    struct Point p;
    p.x = 10;
    p.y = 20;
    return p.x + p.y;
}