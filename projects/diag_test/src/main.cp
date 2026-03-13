struct data { int x; }
bind data { data(i){ host.x = i; } }

int main() {
    let data d1.data(10);
    alias mut data m = d1;
    let data d2 = d1; // Error here: Cannot move borrowed value
    return 0;
}
