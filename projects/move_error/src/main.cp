struct data { int x; }
bind data { data(i){ host.x = i; } }

int main() {
    let data d1.data(10);
    let data d2 = d1; // d1 moves to d2
    printf("%d\n", d1.x); // Error should occur here
    return 0;
}
