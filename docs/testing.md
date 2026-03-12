# 测试验证规范 (Testing & Validation)

为确保 C+ 转译器的正确性，项目采用集成测试与生成的 C 代码比对的验证方案。

## 1. 测试用例集 (Integration Tests)
在 `tests/` 目录下放置 `.cp` 文件，涵盖核心语法：

| 测试文件 | 验证点 |
| :--- | :--- |
| `basic_ownership.cp` | `let` 变量的基础移动与失效检查。 |
| `raii_test.cp` | 作用域结束时 `destroy` 函数的自动插入。 |
| `test_alias.cp` | `alias` 指针映射及移动后的失效拦截。 |
| `fork_remove.cp` | `fork` 结构体字段移除是否正确生成 C 结构。 |
| `clone_fix.cp` | `.clone()` 是否正确生成深拷贝。 |

## 2. 验证流程

### A. 转译验证
- 验证 `cplus build --debug` 能否在 `.temp/` 下生成语义等价的 C 代码。
- 检查 `OwnershipChecker` 是否拦截非法移动（可通过 `test_error.cp` 验证）。

### B. 编译验证
- 验证生成的 C 代码在 GCC 下能够零警告通过编译。

### C. 运行验证
- 运行生成的二进制文件，对比输出结果与预期逻辑（例如 RAII 的创建与销毁日志顺序）。

## 3. 自动化建议
未来建议通过 `cplus test` 命令自动遍历 `tests/` 目录，并比对生成的标准输出 (`.out`) 结果。
