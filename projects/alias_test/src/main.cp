struct data {
    int value;
}

bind data {
    data(v) { host.value = v; }
    void destroy() {}
}

int main() {
    let data d.data(10);
    alias mut data m = d;
    
    // 测试 alias mut 的解引用：m.value 应该转译为 m->value
    m.value = 20;
    
    // 直接参与运算：m 应该转译为 (*m)
    // 注意：目前的 translate_expr 会对 identifier 类型的 alias 自动加 (*)
    // 所以 printf("Value: %d\n", m.value) 
    // 预期：MemberAccess 处理器调用 translate_receiver(m) -> "m"
    // 然后生成 "m->value"
    printf("Value: %d\n", m.value);
    
    return 0;
}
