struct base {
    int x;
}
bind base {
    base(i) { host.x = i; }
    void print() { printf("Base: %d\n", host.x); }
}

fork base as derived {
    + int y;
} bind {
    + derived(i, j) {
        host.x = i;
        host.y = j;
    }
    patch print() as void print() {
        printf("Patched Derived: %d %d\n", host.x, host.y);
    }
}

int main() {
    let derived d.derived(10, 20);
    d.print();
    return 0;
}
