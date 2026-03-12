# 类型系统：struct, bind 与 fork

C+ 强调“数据与行为的分离”，并通过简单的结构化手段实现类型的差异化演变。

## 1. 结构体与绑定 (Struct & Bind)

结构体支持字段默认值，通过 `bind` 块将函数关联到特定类型。

```cpp
struct example {
    int number = 123;
}

bind example {
    example(n) { host.number = n; } // 构造函数 (映射为 example_init)
    void print() { printf("%d\n", host.number); }
    void destroy() { /* 资源释放 */ } // 析构函数 (映射为 example_destroy)
}
```

- **实例化语法**: `let example e.example(10);` (显式初始化语法)
- **host 关键字**: 对应 C 语言中的结构体指针参数，在 `bind` 块内可用。

---

## 2. 结构化演进 (Fork)

`fork` 允许开发者在不修改原结构体的前提下，基于现有结构体生成新类型。

```cpp
fork example as enhanced_example {
    + float score = 0.0; // 添加字段
    - number;            // 移除字段
} bind enhanced_example {
    + void display() { printf("%f\n", host.score); } // 添加方法
    patch example(n) as enhanced_example(n) {
        host.score = (float)n;
    } // 重新实现构造逻辑
}
```

- **极简语义**: 只定义差异，转译器负责生成全新的底层结构体。
- **与 C 互操作**: 生成的新类型依然是标准的 C 结构体。
