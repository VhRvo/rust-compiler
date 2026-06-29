# 本线程总结：Rust 链接 x86-64 汇编

整理时间：2026-06-29

当前问题：在 Apple Silicon Mac 上，用 Rust 调用一段 x86-64 NASM 汇编函数时，直接运行 `rustc stub.rs` 报错：

```text
error[E0570]: "sysv64" is not a supported ABI for the current target
```

## 1. 最终可用命令

当前目录里汇编源文件叫 `compiled_code.s`。

```sh
nasm -f macho64 compiled_code.s -o compiled_code.o
ar rcs libcompiled_code.a compiled_code.o
rustc --target x86_64-apple-darwin stub.rs -L .
./stub
```

验证输出：

```text
Assembly code returned: 37
```

## 2. 问题根因

你的机器默认 Rust target 是 Apple Silicon：

```text
aarch64-apple-darwin
```

但 `stub.rs` 里写的是：

```rust
extern "sysv64" {
    fn start_here() -> i64;
}
```

`sysv64` 是 x86-64 System V ABI，只适合 x86-64 目标。默认用 ARM64 target 编译时，Rust 不支持这个 ABI，所以 `rustc stub.rs` 会失败。

`rustup target add x86_64-apple-darwin` 只是安装 x86-64 macOS target 的标准库，不会切换默认 target。真正使用它，要在 `rustc` 命令里写：

```sh
rustc --target x86_64-apple-darwin stub.rs
```

## 3. `nasm -f macho64 compiled_code.s -o compiled_code.o`

作用：把 NASM 汇编源码编译成 macOS x86-64 object file。

参数解释：

- `nasm`：NASM 汇编器，负责读 NASM 语法的汇编文件。
- `-f macho64`：指定输出格式为 64 位 Mach-O，这是 macOS 使用的 object file 格式。
- `compiled_code.s`：输入汇编源文件。
- `-o compiled_code.o`：指定输出文件名。

为什么不能省略：

- 不写 `-f macho64`，NASM 不会生成 macOS linker 能正常链接的 x86-64 Mach-O object file，后续 Rust 链接会失败或拿到错误格式文件。
- 不写 `-o compiled_code.o`，输出文件名不一定是后续 `ar` 命令期待的 `compiled_code.o`，下一步会找不到文件。
- 输入文件名必须写对。

## 4. `ar rcs libcompiled_code.a compiled_code.o`

作用：把 `compiled_code.o` 打包成 Rust `#[link]` 能找到的静态库。

参数解释：

- `ar`：archive 工具，用来创建或修改 `.a` 静态库。
- `r`：replace/add，把 object file 放进 archive；如果已有同名成员就替换。
- `c`：create，archive 不存在时创建它。
- `s`：写入符号索引，方便 linker 查找函数。
- `libcompiled_code.a`：输出静态库名。
- `compiled_code.o`：输入 object file。

为什么不能省略：

- 不写 `r`，`ar` 没有“放入文件”的操作。
- 不写 `c`，有些环境会在库不存在时给出创建警告；写上更明确。
- 不写 `s`，有些 linker 可能找不到库里的符号；写上就是告诉 `ar` 建好索引。
- 静态库名必须是 `libcompiled_code.a`，因为 Rust 代码里写的是 `#[link(name = "compiled_code", kind = "static")]`。linker 会按 `lib{name}.a` 规则找库。

## 5. `rustc --target x86_64-apple-darwin stub.rs -L .`

作用：把 Rust 代码编译成 x86-64 macOS 程序，并链接当前目录里的静态库。

参数解释：

- `rustc`：Rust 编译器。
- `--target x86_64-apple-darwin`：指定输出程序的平台是 Intel/x86-64 macOS。
- `stub.rs`：输入 Rust 源文件。
- `-L .`：把当前目录加入 native library 搜索路径。

为什么不能省略：

- 不写 `--target x86_64-apple-darwin`，`rustc` 会用默认 target：`aarch64-apple-darwin`，于是 `extern "sysv64"` 报 `E0570`。
- 只运行 `rustup target add x86_64-apple-darwin` 不够，因为那只是安装 target，不是使用 target。
- `-L .` 在当前这个简单目录里可能不写也能过，但不要依赖这个隐式行为。库不在默认搜索路径时，不写它会报找不到 `compiled_code` 库。

## 6. Rust 代码里的 `#[link]`

当前 `stub.rs`：

```rust
#[link(name = "compiled_code", kind = "static")]
extern "sysv64" {
    #[link_name = "\u{1}start_here"]
    fn start_here() -> i64;
}

fn main() {
    let output = unsafe { start_here() };
    println!("Assembly code returned: {}", output);
}
```

`#[link(name = "compiled_code", kind = "static")]` 的意思：

- `name = "compiled_code"`：库名叫 `compiled_code`。
- `kind = "static"`：链接静态库。
- linker 实际会找 `libcompiled_code.a`。

如果静态库叫 `compiled_code.a`、`code.a` 或只有 `compiled_code.o`，这个 `#[link]` 找不到。

## 7. 为什么需要 `#[link_name = "\u{1}start_here"]`

你的 NASM 汇编导出的是：

```asm
global start_here
start_here:
    mov rax, 37
    ret
```

macOS 的 C 符号习惯会涉及前导下划线。Rust 默认可能让 linker 去找：

```text
_start_here
```

但 NASM 实际导出的是：

```text
start_here
```

所以不加 `#[link_name = "\u{1}start_here"]` 时，会出现：

```text
Undefined symbols for architecture x86_64:
  "_start_here"
```

`\u{1}` 的作用是告诉 Rust：这里用原始符号名，不要套默认符号名前缀规则。

## 8. `./stub`

作用：运行当前目录下编译出来的可执行文件。

为什么写 `./`：

- `stub` 是当前目录里的文件。
- shell 默认不会在当前目录找命令。
- 直接写 `stub` 可能报 `command not found`。
- 写 `./stub` 明确表示“运行当前目录下的 stub”。

## 9. 常见失败对照表

| 命令/情况 | 会发生什么 | 原因 |
| --- | --- | --- |
| `rustc stub.rs` | `E0570: "sysv64" is not a supported ABI` | 默认 target 是 `aarch64-apple-darwin` |
| `rustup target add x86_64-apple-darwin` 后再 `rustc stub.rs` | 仍然 `E0570` | 安装 target 不等于使用 target |
| 不生成 `libcompiled_code.a` | Rust 链接找不到库 | `#[link(name = "compiled_code")]` 要找 `libcompiled_code.a` |
| 库名不是 `libcompiled_code.a` | Rust 链接找不到库 | linker 按 `lib{name}.a` 命名规则找 |
| 不加 `#[link_name = "\u{1}start_here"]` | 找不到 `_start_here` | Rust/macOS 符号名和 NASM 导出名不一致 |
| 汇编输出不是 Mach-O x86-64 | linker 无法链接 | macOS x86-64 程序需要 Mach-O 64-bit object |

## 10. 初学者记忆版

这次不是 Rust 版本问题，而是三件事要对齐：

```text
汇编代码架构：x86-64
Rust 编译 target：x86_64-apple-darwin
object file 格式：macho64
```

只要其中一个没对齐，就会在编译或链接阶段失败。
