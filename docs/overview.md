# C+ 语言概览 (C+ Language Overview)

C+ 是一种基于 C 语言构建的系统编程语言。它并非要取代 C 或模仿现代高级语言，而是通过引入**极少量的核心语法扩展**，在保持 C 语言纯粹性的同时，解决了资源管理和结构化组织的痛点。

> "C+ : A systems language with opt-in ownership and structural evolution, designed for seamless migration from C."

## 核心设计：C 的最小扩展

C+ 仅在标准 C 的基础上增加了以下几个关键语法：

1.  **`let` 变量定义**：引入所有权概念，实现自动化的资源回收（RAII）。
2.  **`unsafe` 标记**：显式定义不受所有权托管的 C 原生变量，确保与 C 逻辑的 100% 兼容。
3.  **`bind` 块**：为 `struct` 绑定行为（函数），实现更直观的面向对象风格调用，而不破坏底层结构。
4.  **`alias` 引用**：提供安全的轻量级借用（指针映射），并由编译器进行基本的失效检查。
5.  **`fork` 演化**：支持基于现有结构体生成差异化类型，实现结构化的类型演进。

除了上述特性外，C+ 不打算引入任何其他复杂的语言特性（如泛型、重载、宏系统等），始终保持与 C 语言一致的确定性和高性能。

## 目录索引
- [核心语义：let, unsafe 与 alias](./ownership.md)
- [类型系统：struct, bind 与 fork](./struct_system.md)
- [工具链：项目配置 (cplus.toml) 与 CLI](./tooling.md)
- [开发进度：路线图与状态](./development_plan.md)
