struct example {
    int a = 123;
}

bind example {
    example(n) {
        host.a = n;
    }
    void print_my_number() {
        printf("Number is: %d\n", host.a);
    }
}

int main() {
    // 实例化语法
    let example myexample.example(10);
    myexample.print_my_number();

    // 所有权移动 (Move)
    let example second_example = myexample;
    second_example.print_my_number();

    // 下面这行代码应该导致 cplus 报错
    // myexample.print_my_number(); 

    return 0;
}
