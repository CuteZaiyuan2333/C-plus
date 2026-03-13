struct data {
    int value;
} bind data {
    data(v) { host.value = v; }
    void destroy() {}
}

int main() {
    // 1. 测试借用期间移动 (应该报错)
    let data d1.data(10);
    alias data a1 = d1;
    // let data d2 = d1; // 此处应报错：Cannot move borrowed value: 'd1'

    // 2. 测试多重只读借用 (应该允许)
    let data d3.data(30);
    alias data a2 = d3;
    alias data a3 = d3;
    printf("Read d3: %d, %d\n", a2->value, a3->value);

    // 3. 测试可变借用冲突 (应该报错)
    let data d4.data(40);
    alias mut data m1 = d4;
    // alias data a4 = d4; // 此处应报错：Cannot imm-borrow mutably borrowed value: 'd4'

    // 4. 测试作用域结束释放借用
    let data d5.data(50);
    {
        alias mut data m2 = d5;
        m2->value = 51;
    } 
    let data d6 = d5; // 此处应允许，因为 m2 已失效
    
    return 0;
}
