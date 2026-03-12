struct data {
    int value = 42;
} bind {
    void set_value(v) { host.value = v; }
    void print() { printf("Value: %d\n", host.value); }
}

int main() {
    let data d.data();
    
    // 只读借用
    alias data a = d;
    a.print();
    
    // 可变借用
    alias mut data m = d;
    m.set_value(100);
    m.print();
    
    // 所有权转移
    let data d2 = d; 
    
    // 故意取消注释下面这行来触发 cplus 报错
    // a.print(); 
    
    return 0;
}
