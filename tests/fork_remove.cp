struct base {
    int x;
    int y;
}
fork base as derived {
    - y;
    + int z;
}
int main() {
    let derived d;
    d.x = 1;
    d.z = 2;
    printf("%d %d\n", d.x, d.z);
    return 0;
}
