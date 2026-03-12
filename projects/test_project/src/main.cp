#include <time.h>

struct data {
    int value = 42;
} bind data {
    data() {}
    void set_value(v) { host.value = v; }
    void print() { printf("Value: %d\n", host.value); }
}

// 验证 fork 演化
fork data as advanced_data {
    + int timestamp;
} bind advanced_data {
    patch data() as advanced_data() {
        host.value = 777;
        host.timestamp = (int)time(NULL);
    }
    void show_time() {
        printf("Created at: %d, Value: %d\n", host.timestamp, host.value);
    }
}

int main() {
    let advanced_data ad.advanced_data();
    ad.show_time();

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
    
    // 验证所有权检查是否依然有效 (若取消注释应报错)
    // a.print(); 
    
    return 0;
}
