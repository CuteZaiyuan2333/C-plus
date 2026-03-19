struct Point {
    int x = 0;
    int y = 0;
}

bind Point {
    Point(x, y) {
        host.x = x;
        host.y = y;
        printf("Point created at (%d, %d)\n", host.x, host.y);
    }
    void destroy() {
        printf("Point at (%d, %d) destroyed\n", host.x, host.y);
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
    {
        let Point p.Point(10, 20);
        p.print();
        
        let Point3D p3.Point3D(30, 40);
        p3.z = 99;
        p3.print3d();
        
        // Move semantics
        let Point p2 = p;
        p2.print();
        // p is no longer accessible here
    }
    printf("End of scope\n");
    return 0;
}