# 工具链：项目配置 (cplus.toml) 与 CLI

C+ 转译器集成了简易的包管理和构建系统，通过 `cplus.toml` 管理编译。

## 1. 项目配置 (cplus.toml)

项目根目录下的配置文件用于指示 C+ 如何转译并调用 GCC 链接。

```toml
[package]
name = "my_app"      # 项目名称 (生成的 exe 名称)
version = "0.1.0"
type = "bin"        # 支持 bin (默认), staticlib, dylib

[build]
flags = ["-O2"]     # 全局编译参数 (CFLAGS)
includes = ["inc"]  # 包含路径 (-I)
lib_dirs = ["libs"] # 库路径 (-L)
libs = ["m"]        # 链接库 (-l)

[profile.debug]
flags = ["-g", "-DDEBUG"]
opt_level = 0

[profile.release]
flags = ["-DNDEBUG"]
opt_level = 3

[dependencies]
# 系统库支持 (pkg-config)
openssl = { system = true, version = "1.1" }
```

---

## 2. 命令行工具 (CLI)

`cplus` 工具通过 Rust 编写，集成了转译与编译全流程。

- **`cplus init <name>`**: 初始化新项目，自动生成 `cplus.toml` 和 `src/main.cp`。
- **`cplus build [--debug]`**:
    1.  扫描 `src/` 下的所有 `.cp`。
    2.  转译为 `.c` 存放于 `.temp/`。
    3.  根据 `cplus.toml` 参数聚合 GCC 命令进行最终编译。
- **`cplus run [--debug]`**: 编译并立即运行。

---

## 3. 构建产物结构
- **`src/`**: C+ 源代码 (`.cp`)。
- **`.temp/`**: 转译生成的中间 C 代码。
- **`build/`**: 最终二进制产物，按 `debug/release` 区分。
