struct Point {
    int x = 0;
} bind Point {
    Point(x) { host.x = x; }
    Point clone() {
        let Point new_p.Point(host.x);
        return new_p;
    }
}

int main() {
    let Point p.Point(10);
    let Point p2 = p.clone(); 
    p.x = 20; 
    printf("p.x = %d, p2.x = %d\n", p.x, p2.x);
    return 0;
}
