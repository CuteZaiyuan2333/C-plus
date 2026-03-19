# C+ (cplus) - 极简高性能 C 语言扩展

C+ 是一种基于 C 语言构建的系统编程语言。它并非要取代 C 或模仿现代高级语言，而是通过引入**极少量的核心语法扩展**，在保持 C 语言纯粹性的同时，解决了资源管理、内存安全和结构化组织的痛点。

> "C+ : A systems language with opt-in ownership and structural evolution, designed for seamless migration from C."
>
> [!IMPORTANT]
> **项目重心变更**：团队已决定放弃 LLVM 原生后端集成，全力转向更稳定的**转译器（Transpiler）**开发，以确保与 C 语言生态的最佳兼容性与系统稳定性。

## 🚀 核心特性

- **所有权系统 (`let`)**：引入 Move 语义，实现自动化的资源回收（RAII），消除内存泄漏。
- **100% C 兼容 (`unsafe`)**：显式定义原生 C 变量，支持现有 C 代码库的无缝迁移。
- **行为绑定 (`bind`)**：为 `struct` 绑定方法，实现直观的面向对象风格调用，而不破坏底层 C 结构。
- **安全引用 (`alias`)**：提供带编译器失效检查的轻量级借用（指针映射）。
- **结构演进 (`fork`)**：支持基于现有结构体生成差异化类型，实现结构化的类型演化。

## 🛠️ 工具链集成

C+ 内置了基于 Rust 编写的自动化构建工具 `cplus`，集成了转译、编译与运行全流程。

### 常用命令
- `cplus init <name>`：初始化新项目，自动生成 `cplus.toml`。
- `cplus build [--debug]`：扫描 `src/` 下的 `.cp` 文件，转译并调用 GCC 编译。
- `cplus run [--debug]`：一键编译并立即运行。

## 📝 快速上手

在 `src/main.cp` 中体验 C+ 的核心魅力：

```cpp
struct Resource {
    int id;
}

bind Resource {
    // 构造函数
    Resource(id) {
        host.id = id;
        printf("Resource %d initialized\n", host.id);
    }
    // 析构函数 (RAII)
    void destroy() {
        printf("Resource %d destroyed automatically\n", host.id);
    }
}

int main() {
    // 使用 let 托管资源，作用域结束自动调用 destroy
    let Resource r.Resource(1024);
    
    // 使用 alias 借用资源（只读别名）
    alias Resource a = r;
    printf("Accessing resource ID: %d\n", a->id);

    return 0;
}
```

## 📂 项目结构

- **`cplus/`**：核心转译器实现（基于 Rust）。
- **`docs/`**：详细的设计文档与开发计划。
- **`projects/`**：示例项目与集成测试案例。
- **`tests/`**：核心语法验证集。
- **`.temp/`**：转译生成的中间 C 代码（构建时产生）。
- **`build/`**：最终二进制产物。

## 📈 开发路线图

目前已完成核心语义定义、所有权追踪、RAII 自动化及构建工具。未来计划包括：
- [ ] 优化 `alias mut` 到 C 指针的转换逻辑。
- [ ] 增强 `fork` 对复杂结构体嵌套的解析。
- [ ] 持续优化转译器稳定性与 C 兼容性。
- [ ] 完善标准库跨平台支持。

## 📜 许可

本项目遵循 MIT 开源许可。
