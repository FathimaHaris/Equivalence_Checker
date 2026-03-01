int classify(int x) {
    if (x < 0) {
        return -1;
    } else if (x == 0) {
        return 0;
    } else if (x < 10) {
        return 1;
    } else if (x < 100) {
        return 2;
    } else {
        return 3;
    }
}