# C+ LLVM 编译器开发路线图 (LLVM & Clang 集成)

本阶段的目标是将 C+ 从一个“C 转译器”演进为一个真正的“原生编译器”。C+ 将作为 C 语言的严格超集，直接利用 LLVM 生成高性能机器码，并集成 `libclang` 以实现对标准 C 语言（`.c`/.`.h`）的 100% 兼容。

---

## 核心架构设计

### 1. 双模前端 (Dual-Mode Frontend)
根据文件后缀切换解析策略：
- **C 兼容模式 (.c / .h)**: 调用 `libclang` 解析 AST，将所有变量定义隐式标记为 `unsafe`。
- **C+ 扩展模式 (.cp / .cph)**: 使用自研解析器处理 `let`, `alias`, `bind`, `fork` 等语法，强制执行所有权检查。

### 2. 统一语义层 (Unified Semantic Layer)
- **Ownership Checker**: 对 `let` 变量执行移动语义和生命周期失效检查。
- **Type System**: 将 C+ 的 `struct ... bind` 映射为扁平化的 C 结构体和关联函数。

### 3. LLVM 后端 (LLVM IR Generation)
- **原生二进制生成**: 摆脱中间 C 代码，直接生成 LLVM IR。
- **自动析构注入**: 在 IR 层面，根据 `let` 变量的作用域终点自动插入 `_destroy` 函数调用。

---

## 开发阶段划分

### 第一阶段：libclang 集成与隐式 Unsafe (当前重点)
- [ ] **环境搭建**: 在 C+ 构建系统中引入 LLVM/Clang 开发库。
- [ ] **C 解析桥接**: 实现一个 C+ 模块，通过 `libclang` 的 C API 遍历标准 C 文件的抽象语法树。
- [ ] **语义对齐**: 确保 `.c` 文件中的变量声明在 C+ 编译器内部被视为 `OwnershipStatus::Unsafe`，使其无需修改即可通过编译。

### 第二阶段：LLVM 后端基础
- [ ] **IR 生成器**: 实现基础的 `LLVMGenerator`，支持基本类型、算术运算和函数调用。
- [ ] **结构体映射**: 将 `struct` 映射为 `llvm::StructType`，将 `bind` 方法映射为以 `host` 指针为首个参数的函数。
- [ ] **符号表统一**: 整合 `libclang` 生成的符号与 C+ 自有的符号表。

### 第三阶段：所有权与 RAII 落地
- [ ] **生命周期标注**: 在 LLVM IR 中插入生命周期标记。
- [ ] **析构逻辑注入**: 在 `let` 变量超出作用域的每一条退出路径（包括 `return`）自动生成析构调用指令。
- [ ] **移动语义优化**: 利用 `let` 的唯一性，向 LLVM 优化器传递 `noalias` 元数据。

### 第四阶段：自举 (Self-Hosting)
- [ ] **标准库封装**: 用 C+ 编写 LLVM C API 的安全包装（利用 RAII 托管 LLVM 资源）。
- [ ] **编译器迁移**: 开始用 C+ 重新编写解析器、检查器和生成器。
- [ ] **闭环测试**: 使用 C+ 编写的编译器编译其自身。

---

## 关键技术细节：libclang 的角色

为了保证对 C 的 100% 兼容，我们不会自研 C 解析器。
1. **Header Parsing**: 当 `.cp` 文件中出现 `#include` 时，调用 `libclang` 解析该头文件，并将导出的符号（函数原型、结构体定义）导入 C+ 符号表。
2. **Implicit Unsafe**: 
   ```cpp
   // 在 .c 文件中
   int x = 10; // libclang 解析为 VarDecl -> 编译器标记为 unsafe
   
   // 在 .cp 文件中
   int x = 10; // 报错：必须使用 let 或 unsafe
   ```
3. **互操作性**: C+ 代码可以自由调用 `.c` 文件中定义的函数，反之亦然，因为它们在 LLVM IR 层面具有相同的链接规范（C Calling Convention）。

---

## 长期目标：C+ 系统编程平台
- 摆脱对系统安装的 GCC/Clang 驱动程序的依赖，成为独立的工具链。
- 提供超越 C 语言的性能优化潜力（基于所有权信息的别名分析）。
- 保持极简的语法扩展，不引入 OOP 的复杂性，仅通过“行为绑定”实现模块化。
