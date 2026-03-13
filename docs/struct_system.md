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
## 2. 结构复刻与演进 (Fork)

`fork` 是一个极简的**结构体生成器**，允许开发者基于现有结构体快速克隆并差异化定制新类型，而无需手动复制字段。

```cpp
fork example as enhanced_example {
    + float score = 0.0; // 增加新字段
    - number;            // 移除不需要的字段
}
```

- **静态复刻 (Static Cloning)**: `fork` 不属于 OOP 意义上的“类继承”。它在转译阶段生成一个**完全独立、扁平化**的 C 结构体。
- **差异化定义**: 只需声明与基类的差异 (`+` 或 `-`)，转译器会自动聚合所有字段。
- **0 运行时开销**: 由于不涉及虚表或动态绑定，生成的代码与手写 C 结构体性能完全一致。
- **与 C 互操作**: 生成的新类型依然是标准的 C 结构体，可直接用于原生 C 库调用。

