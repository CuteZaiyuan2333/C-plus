struct Point {
    int x = 0;
} bind Point {
    Point(x) { host.x = x; }
}
int main() {
    let Point p.Point(10);
    let Point p2 = p;
    p.x = 20; // Should error here
    return 0;
}
