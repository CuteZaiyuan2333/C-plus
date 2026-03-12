struct Point {
    int x = 0;
    int y = 0;
}

bind Point {
    Point(x, y) {
        host.x = x;
        host.y = y;
    }
    void destroy() {
        // Points don't really need cleanup, but RAII will call this
    }
    void print() {
        printf("Point(%d, %d)\n", host.x, host.y);
    }
}

fork Point as Point3D {
    + int z = 100;
} bind {
    + void print3d() {
        printf("Point3D(%d, %d, %d)\n", host.x, host.y, host.z);
    }
}

int main() {
    let Point p.Point(10, 20);
    p.print();

    let Point3D p3.Point3D(30, 40);
    p3.x = 50;
    p3.print3d();

    let Point p2 = p; // p is moved here
    // p.print(); // This should fail if transpiler checks correctly

    unsafe Point* up = &p2;
    up->print();

    return 0;
}
