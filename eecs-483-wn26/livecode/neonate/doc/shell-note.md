# Shell 脚本笔记

这份笔记解释两个脚本：

- `asmToExe.sh`：输入 x86-64 汇编文件，输出可执行文件 `main.exe`。
- `neonateToExe.sh`：输入 Neonate 源文件，先生成汇编，再输出并运行 `main.exe`。

## 命令和缩写速查

| 写法 | 展开 / 含义 | 在脚本里的作用 |
| --- | --- | --- |
| `asm` | assembly，汇编代码 | `asmToExe.sh` 的输入类型 |
| `exe` | executable file，可执行文件 | `main.exe` 是输出文件 |
| `env` | environment，环境 | 从 `PATH` 里找 `bash` |
| `bash` | Bourne Again Shell | 运行 shell 脚本 |
| `uname` | Unix name | 打印系统信息 |
| `uname -s` | `-s` 表示 system name / kernel name | 只打印系统名称，比如 `Darwin` 或 `Linux` |
| `OS` | operating system，操作系统 | 脚本按操作系统选择输出格式 |
| `x86_64` | x86 64-bit architecture，64 位 x86 架构 | 目标机器架构 |
| `macho64` | 64-bit Mach-O | macOS 的 64 位目标文件格式 |
| `elf64` | 64-bit Executable and Linkable Format | Linux 的 64 位目标文件格式 |
| `nasm` | Netwide Assembler | 把汇编代码编译成目标文件 |
| `nasm -f` | format，格式 | 指定输出格式 |
| `nasm -o` | output，输出 | 指定输出文件名 |
| `ar` | archiver，归档工具 | 生成静态库 |
| `ar r` | replace / insert | 插入或替换归档里的文件 |
| `ar u` | update | 只更新较新的文件 |
| `ar s` | symbol index | 生成符号索引，方便链接器查找 |
| `ld` | linker / link editor，链接器 | 最终把代码和库链接成可执行文件；这里由 `rustc` 间接调用 |
| `rustc` | Rust compiler | 编译 Rust 文件并完成链接 |
| `rustc -L` | library search path | 指定库搜索目录；`-L .` 表示当前目录 |
| `rustc -o` | output，输出 | 指定输出文件名 |
| `rustc --target` | target platform，目标平台 | 指定编译目标 |
| `Cargo` | Rust package manager and build tool | 构建 Neonate 编译器 |
| `cargo build --release` | release build，发布构建 | 生成优化版本程序 |
| `mkdir` | make directory，创建目录 | 创建 `build` 目录 |
| `mkdir -p` | parents / no error if exists | 创建父目录，目录已存在也不报错 |
| `cat` | concatenate，连接并输出 | 打印文件内容 |
| `cd` | change directory，切换目录 | 进入 `build` 目录 |
| `PID` | process identifier，进程编号 | `$$`、`$!` 相关 |

## Shell 语法速查

常见特殊变量：

```bash
$0    当前脚本或命令的名字
$1    第 1 个参数
$2    第 2 个参数
$#    参数个数
$@    所有参数，保留每个参数的边界
$*    所有参数，通常拼成一个整体字符串
$?    上一个命令的退出状态码
$$    当前 shell 进程的 PID
$!    最近一个后台进程的 PID
```

`if` 判断的是后面命令的退出状态码：

```bash
0     表示成功 / true
非 0  表示失败 / false
```

所以：

```bash
if [ "$os_type" = "Darwin" ]; then
    echo "macOS"
fi
```

意思是：运行 `[ "$os_type" = "Darwin" ]` 这个测试命令；如果它返回 `0`，就执行 `then`。

`[` 和 `test` 基本等价：

```bash
if [ "$os_type" = "Darwin" ]; then
if test "$os_type" = "Darwin"; then
```

因为脚本使用的是 `bash`，也可以写成：

```bash
if [[ "$os_type" == "Darwin" ]]; then
```

变量展开推荐加引号：

```bash
if [ "${os_type}" = "Darwin" ]; then
nasm -f "${format}" -o compiled_code.o "$1"
```

原因：

- `"${os_type}"`：变量为空时也不会破坏测试表达式。
- `"$1"`：输入文件名有空格时不会被拆开。
- `${name}`：花括号只是写清变量名边界，和 `$name` 取的是同一个变量。

其他语法：

```bash
cmd1 && cmd2
```

`&&` 表示 `cmd1` 成功才运行 `cmd2`。

```bash
long command \
 && next command
```

行尾 `\` 表示这一行还没结束，下一行继续。

```bash
./target/release/neonate "$1" > build/compiled_code.s
```

`>` 表示把标准输出写入文件，会覆盖原文件。

```bash
./main.exe
```

`./` 表示运行当前目录下的程序。

## 两个脚本的关系

`asmToExe.sh` 的输入已经是汇编代码：

```text
assembly file
  -> nasm
  -> object file
  -> static library
  -> rustc + stub.rs
  -> executable file
```

`neonateToExe.sh` 多做了第一步：先构建并运行 Neonate 编译器，把 `.neonate` 源文件变成汇编代码：

```text
Neonate source file
  -> cargo build --release
  -> target/release/neonate
  -> assembly file
  -> nasm
  -> object file
  -> static library
  -> rustc + stub.rs
  -> executable file
  -> run executable file
```

`stub.rs` 是 Rust 写的运行外壳。它会链接汇编生成的静态库，调用汇编里提供的 `start_here` 函数，并打印返回值。

## 公共部分

两个脚本开头基本一样：

```bash
#!/usr/bin/env bash

os_type=$(uname -s)

if [ $# -ne 1 ]; then
    echo "Usage: $0 <input_file>"
    exit 1
fi

if [ "$os_type" = "Darwin" ]; then
    format="macho64"
    extra_args="--target=x86_64-apple-darwin"
elif [ "$os_type" = "Linux" ]; then
    format="elf64"
else
    echo "unknown platform"
    exit 1
fi
```

含义：

- `#!/usr/bin/env bash`：用 `env` 在环境变量 `PATH` 里找到 `bash`，再用 `bash` 运行脚本。
- `os_type=$(uname -s)`：运行 `uname -s`，把系统名称保存到 `os_type`。
- `[ $# -ne 1 ]`：检查参数个数是不是不等于 1。
- `echo "Usage: $0 <input_file>"`：参数数量不对时打印用法。
- `exit 1`：以失败状态退出脚本。
- `Darwin`：macOS 的系统名称。
- `Linux`：Linux 的系统名称。
- `format="macho64"`：macOS 上让 Netwide Assembler 输出 64 位 Mach-O 目标文件。
- `format="elf64"`：Linux 上让 Netwide Assembler 输出 64 位 ELF 目标文件。
- `extra_args="--target=x86_64-apple-darwin"`：macOS 上给 Rust compiler 指定目标平台。

## asmToExe.sh 特有流程

使用方式：

```bash
./asmToExe.sh your_file.asm
./main.exe
```

核心命令：

```bash
nasm -f $format -o compiled_code.o $1 \
 && ar rus libcompiled_code.a compiled_code.o \
 && rustc stub.rs -L . $extra_args -o main.exe
```

逐步含义：

1. `nasm -f $format -o compiled_code.o $1`
   用 Netwide Assembler 把输入汇编文件 `$1` 编译成目标文件 `compiled_code.o`。

2. `ar rus libcompiled_code.a compiled_code.o`
   用 archiver 把目标文件打包成静态库 `libcompiled_code.a`。

3. `rustc stub.rs -L . $extra_args -o main.exe`
   用 Rust compiler 编译 `stub.rs`，在当前目录找静态库，并输出 `main.exe`。

这个脚本不会自动运行 `main.exe`，需要手动执行：

```bash
./main.exe
```

## neonateToExe.sh 特有流程

使用方式：

```bash
./neonateToExe.sh your_file.neonate
```

核心命令：

```bash
cargo build --release \
 && echo "successfully built compiler" \
 && mkdir -p build \
 && ./target/release/neonate $1 > build/compiled_code.s \
 && echo "Generated assembly:" \
 && cat build/compiled_code.s \
 && cd build \
 && nasm -f $format -o compiled_code.o compiled_code.s \
 && ar rus libcompiled_code.a compiled_code.o \
 && rustc ../stub.rs -L . $extra_args -o main.exe \
 && ./main.exe
```

逐步含义：

1. `cargo build --release`
   用 Cargo 构建优化版本的 Neonate 编译器。

2. `mkdir -p build`
   创建 `build` 目录；目录已存在也不报错。

3. `./target/release/neonate $1 > build/compiled_code.s`
   运行 Neonate 编译器，把输入源文件 `$1` 编译成汇编，并写入 `build/compiled_code.s`。

4. `cat build/compiled_code.s`
   打印生成的汇编代码，方便检查。

5. `cd build`
   进入 `build` 目录。后续生成的 `compiled_code.o`、`libcompiled_code.a`、`main.exe` 都在这里。

6. `nasm -f $format -o compiled_code.o compiled_code.s`
   把汇编文件编译成目标文件。

7. `ar rus libcompiled_code.a compiled_code.o`
   把目标文件打包成静态库。

8. `rustc ../stub.rs -L . $extra_args -o main.exe`
   编译上一层目录的 `stub.rs`，链接当前目录里的静态库，输出 `main.exe`。

9. `./main.exe`
   运行生成的可执行文件。
