struct data { int x; }
bind data { data(i){ host.x = i; } }

int main() {
    let data d1.data(10);
    let data d2 = d1.clone(); // d1 is cloned to d2, d1 stays active
    printf("%d\n", d1.x); // Should work now
    return 0;
}
