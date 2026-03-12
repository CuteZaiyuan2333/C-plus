# 核心语义：let, unsafe 与 alias

C+ 的资源管理核心在于显式区分**所有权**与**原生 C 变量**。

## 1. 变量定义的两种形态

### 托管模式 (`let`)
通过 `let` 定义的变量受到 C+ 生命周期管理，这是实现自动资源回收（RAII）的唯一途径。

```cpp
let int a = 10;
let MyResource r.init(100);
```

- **自动析构**：作用域结束时，C+ 编译器自动插入该类型的析构调用（如 `MyResource_destroy`）。
- **所有权移动 (Move)**：当赋值给另一个 `let` 变量时，原变量失效。

### 原生模式 (`unsafe`)
`unsafe` 关键字用于标记不受所有权托管的变量，这正是 C+ 兼容标准 C 语言的核心机制。

```cpp
unsafe int* p = &some_c_val;
unsafe int x = 5;
```

- **C 语言迁移**：将 `.c` 改为 `.cp` 后，只需在所有变量定义前增加 `unsafe` 前缀，代码即可无缝迁移到 C+ 编译器。

---

## 2. 借用与引用 (`alias`)

`alias` 提供了一种显式的、带编译器失效检查的引用机制，在底层实现上它表现为指针。

```cpp
let MyData d.init();
alias MyData a = d;     // 只读别名 (const T*)
alias mut MyData m = d; // 可变别名 (T*)
```

- **失效拦截**：若 `d` 的所有权发生转移（Move），则后续通过 `a` 或 `m` 的访问将在编译阶段报错。

---

## 3. 显式克隆 (`clone`)

针对 `let` 托管变量，C+ 提供了 `.clone()` 内置方法以执行深拷贝并保留原所有权，这通常由编译器根据 `struct` 字段自动生成。
