struct demo { int a = 1; }
bind demo { demo(n){ host.a = n; } }
int main() { let demo d.demo(10); return 0; }