# 本线程总结：macOS 上运行 NASM Hello World

整理时间：2026-07-01

当前问题：照着 NASM 教程运行 `Nasm/Hello/hello.asm`，Linux 版本和 macOS 版本都失败。

本机环境：

```text
CPU: arm64 Apple Silicon
NASM: /opt/homebrew/bin/nasm
Apple linker: /usr/bin/ld
```

## 1. 最终可用命令

在源码目录运行：

```sh
cd /Users/zhangyongzhuo/Documents/EECS483/Nasm/Hello
mkdir -p build
nasm -fmacho64 hello.asm -o build/hello.o
/usr/bin/ld -static -arch x86_64 -platform_version macos 11.0 $(xcrun --show-sdk-version) -e start build/hello.o -o build/hello
./build/hello
```

验证输出：

```text
Hello, World
```

## 2. 问题根因

NASM 教程第一页讲的是 x86-64 汇编，不是 Apple Silicon 的 ARM64 汇编。

你的 Mac 是 `arm64`，但 `hello.asm` 用的是 x86-64 寄存器和指令：

```asm
rax
rdi
rsi
syscall
```

所以这里生成的是 `x86_64` Mach-O 程序，在 Apple Silicon 上通过 Rosetta 运行。

另一个坑是 linker。普通 `ld` 可能被 PATH 解析到 Anaconda 的 linker，不一定是 Apple 的 linker，所以命令里显式使用：

```sh
/usr/bin/ld
```

现代 macOS 的 `ld hello.o` 也不够，通常会遇到：

```text
ld: Missing -platform_version option
```

如果不加 `-static`，linker 还会默认走 C runtime/dyld 入口，开始找 `_main`，但这个程序只有 `start`，没有 `_main`。

## 3. 汇编命令

```sh
nasm -fmacho64 hello.asm -o build/hello.o
```

参数解释：

- `nasm`：NASM 汇编器。
- `-f macho64` / `-fmacho64`：输出 64 位 Mach-O object file，macOS 用这个格式。
- `hello.asm`：输入汇编源码。
- `-o build/hello.o`：把 object file 输出到 `build/` 目录。

这一步只生成 object file，还不是可执行程序。

## 4. 链接命令

```sh
/usr/bin/ld -static -arch x86_64 -platform_version macos 11.0 $(xcrun --show-sdk-version) -e start build/hello.o -o build/hello
```

逐段解释：

- `/usr/bin/ld`：Apple 自带 linker，避免用到 Anaconda 或其他 PATH 里的 `ld`。
- `-static`：生成不依赖 dynamic linker/C runtime 的可执行文件，适合这个直接 syscall 的例子。
- `-arch x86_64`：输出 x86-64 程序。
- `-platform_version macos 11.0 ...`：告诉现代 macOS linker 目标平台、最低系统版本、SDK 版本。
- `$(xcrun --show-sdk-version)`：让 shell 先运行 `xcrun --show-sdk-version`，把输出的 SDK 版本填进命令。
- `-e start`：指定入口点是 asm 里的 `start` label。
- `build/hello.o`：输入 object file。
- `-o build/hello`：把可执行文件输出到 `build/` 目录。

## 5. `xcrun` 是什么

`xcrun` 是 Xcode / Command Line Tools 带的工具查找器。

它用来按当前 Xcode/Command Line Tools 配置找到正确的 Apple 工具和 SDK 信息。

这里用的是：

```sh
xcrun --show-sdk-version
```

意思是显示当前 macOS SDK 版本，比如：

```text
15.5
```

所以这段：

```sh
$(xcrun --show-sdk-version)
```

会被 shell 替换成类似：

```text
15.5
```

`xcrun` 可以粗略理解为 `Xcode run`，其中 `xc` 指 Xcode。

## 6. 核心 `hello.asm` 内容

```asm
global start
section .text
start:
    mov rax, 0x02000004
    mov rdi, 1
    mov rsi, message
    mov rdx, 13
    syscall
    mov rax, 0x02000001
    xor rdi, rdi
    syscall

section .data
message:
    db "Hello, World", 10
```

关键点：

- macOS x86-64 的 `write` syscall 是 `0x02000004`。
- macOS x86-64 的 `exit` syscall 是 `0x02000001`。
- 入口 label 是 `start`，所以 linker 用 `-e start`。

## 7. Git 忽略规则

把编译输出统一放进 `build/`，然后 `.gitignore` 用通用规则忽略：

```gitignore
build/
**/build/
```

这样 `build/hello.o` 和 `build/hello` 都不会进 git，不需要为每个无扩展名可执行文件单独写规则。

Makefile 版本见 [makefile-notes.md](makefile-notes.md)。
