struct data {
    int x;
}

bind data {
    data(i) { host.x = i; }
    void destroy() {}
}

int main() {
    let data d1.data(10);
    let data d2 = d1;
    
    // 故意触发错误：使用已移动的变量
    printf("%d\n", d1.x); 
    
    return 0;
}
