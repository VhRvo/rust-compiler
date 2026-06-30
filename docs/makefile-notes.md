# Makefile Notes

整理时间：2026-06-29

目标：用正常、传统的 Makefile 写法构建 Neonate，并理解当前 Makefile 里每个“不熟悉”的符号。

## 1. 正常用法：用变量传参

推荐：

```sh
make run FILE=source/2021.int
```

也可以直接：

```sh
make run
```

因为 Makefile 里有默认值：

```make
FILE ?= source/2021.int
```

含义：

- 如果命令行没有传 `FILE=...`，就用 `source/2021.int`。
- 如果命令行传了 `FILE=source/foo.int`，就用传入的文件。

不要用：

```sh
make run source/2021.int
```

Make 会把它理解成两个 target：

```text
target 1: run
target 2: source/2021.int
```

所以可能出现：

```text
make: Nothing to be done for `source/2021.int'
```

Best practice：Make 传参数用变量，比如 `FILE=...`。

如果不用这个方式，就要解析 `MAKECMDGOALS`，那是技巧，不适合初学阶段。

## 1.1. Make 变量和 shell 变量

Make 变量和 shell 变量不是一回事。

两种传法不同：

```sh
FILE=source/2021.int make run
```

这是 shell 语法：给 `make` 进程临时传环境变量。Make 会导入它，但普通 Makefile 赋值可以覆盖它。

比如 Makefile 里写了 `FILE = source/default.int`，那么 `FILE=source/2021.int make run` 通常还是会用 `source/default.int`。如果想让外面传入的 `FILE` 生效，用默认赋值：

```make
FILE ?= source/default.int
```

```sh
make run FILE=source/2021.int
```

这是 Make 命令行变量，优先级更高，通常会覆盖 Makefile 里的普通赋值。

简单记：

```text
FILE=... make run    # shell 环境变量，优先级较低
make run FILE=...    # Make 命令行变量，优先级较高
```

Best practice：用户传给 Make 的参数写成 `make run FILE=...`。`$(FILE)` 和 `$$FILE` 的区别见第 2 节。

## 2. Make 展开和 `$`

Makefile 不是直接交给 shell 跑。Make 先展开自己的变量、函数、自动变量，再把命令交给 shell。

```make
$(OBJ): $(ASM)
	nasm -f $(NASM_FORMAT) -o '$@' '$<'
```

可能先展开成：

```sh
nasm -f macho64 -o 'build/2021/2021.o' 'build/2021/2021.s'
```

常用写法：

```make
$(FILE)   # Make 变量，多个字符要用括号
$@        # 当前目标
$<        # 第一个依赖
$^        # 所有依赖
$$HOME    # 留给 shell，最终 shell 看到 $HOME
```

`$$HOME` 里，Make 只把 `$$` 变成一个普通 `$`，后面的 `HOME` 字符串原样保留，所以 shell 最后看到 `$HOME`。这里 `$` 不是通用转义符；`$$` 只是 Make 里用来生成字面量 `$` 的特殊写法。

不要写 `$FILE` 表示 Make 变量 `FILE`；Make 会把它当成 `$F` 加普通文本 `ILE`。也不要写 `$HOME` 表示 shell 变量；recipe 里 shell 变量要写 `$$HOME`。

Best practice：Make 变量用 `$(VAR)`，Make 自动变量用 `$@` / `$<` / `$^`，shell 变量用 `$$VAR`。

## 3. 当前构建链路

当前 Makefile 把所有生成物放进 `build/`，避免污染 git。

```text
source/2021.int
-> build/2021/2021.s
-> build/2021/2021.o
-> build/2021/libcompiled_code.a
-> build/2021/2021.run
```

对应规则：

```make
$(ASM): $(ABS_FILE) src/main.rs Cargo.toml
	@mkdir -p '$(OUT_DIR)'
	CARGO_TARGET_DIR='$(CARGO_TARGET_DIR)' cargo run --quiet -- '$(ABS_FILE)' > '$@'

$(OBJ): $(ASM)
	nasm -f $(NASM_FORMAT) -o '$@' '$<'

$(LIB): $(OBJ)
	ar rcs '$@' '$<'

$(RUNNER): $(LIB) stub.rs
	rustc --target $(RUST_TARGET) stub.rs -L '$(OUT_DIR)' -o '$@'
```

Best practice：生成物集中放进 `build/`，并在 `.gitignore` 忽略。

如果不这样，`.s`、`.o`、`.a`、`.run` 会散在项目根目录，git status 很乱。

## 4. Make 变量赋值

例子：

```make
BUILD_DIR := build
RUST_TARGET := x86_64-apple-darwin
```

`:=` 是立即赋值。右边会在这一行就展开。

```make
A = $(B)
B = hello
```

`=` 是延迟展开赋值。右边先原样保存，等到使用 `$(A)` 时再展开，所以这里 `$(A)` 会得到 `hello`。

shell 变量没有这种延迟展开：`A=$B` 会在赋值当时展开 `$B`。

```make
FILE ?= source/2021.int
```

`?=` 是默认赋值。只有 `FILE` 没有被设置时才生效。

```make
CFLAGS = -Wall
CFLAGS += -O2
```

`+=` 是追加赋值，把一段文本追加到变量后面，中间通常自动加一个空格。Make 变量本质上都是文本，不需要声明成字符串。

使用变量：

```make
$(BUILD_DIR)
$(FILE)
```

Best practice：

- 固定配置用变量，比如 `RUST_TARGET`、`NASM_FORMAT`。
- 可被命令行覆盖的输入用 `?=`，比如 `FILE ?= source/2021.int`。

如果到处硬编码路径，以后改文件名或 target 会改很多地方。

## 5. 从输入文件名生成输出名

```make
NAME := $(basename $(notdir $(FILE)))
```

这里的 `notdir` 和 `basename` 是 Make 内置函数，不是 shell 命令。

假设：

```make
FILE := source/2021.int
```

先执行：

```make
$(notdir $(FILE))
```

结果：

```text
2021.int
```

再执行：

```make
$(basename 2021.int)
```

结果：

```text
2021
```

所以：

```make
NAME := 2021
```

后面就可以生成：

```make
OUT_DIR := build/2021
ASM := build/2021/2021.s
OBJ := build/2021/2021.o
RUNNER := build/2021/2021.run
```

Best practice：用 Make 内置函数推导输出路径，避免手动写死 `2021`。

如果写死 `2021`，以后换成 `source/42.int` 时 Makefile 也要改。

## 6. 绝对路径

```make
ABS_FILE := $(abspath $(FILE))
```

`abspath` 是 Make 内置函数，用来把路径转成绝对路径。

如果：

```make
FILE := source/2021.int
```

那么 `ABS_FILE` 类似：

```text
/Users/zhangyongzhuo/Documents/EECS483/Neonate/source/2021.int
```

这里用绝对路径是为了让 `cargo run` 读取输入文件时更稳，不受当前目录变化影响。

Best practice：当命令可能改变工作目录，或者工具内部再启动程序时，传绝对路径更稳。

如果只传相对路径，工具的工作目录一变，就可能找不到文件。

## 7. 规则、依赖和自动变量

规则格式：

```make
目标: 依赖1 依赖2
	命令
```

命令行前面必须是 tab。Make 会根据目标和依赖的时间判断是否重建。

常用自动变量：

```make
$@   # 当前目标
$<   # 第一个依赖
$^   # 所有依赖，去重
$+   # 所有依赖，不去重
```

例子：

```make
$(OBJ): $(ASM)
	nasm -f $(NASM_FORMAT) -o '$@' '$<'
```

Best practice：真实文件写依赖，命令里用 `$@`、`$<`、`$^`，少手写重复路径。

## 8. 第 N 个依赖

Make 没有 `$2`、`$3` 这种自动变量。

如果要取第 2、第 3、第 4 个依赖，可以用 `word`：

```make
$(word 2,$^)
$(word 3,$^)
$(word 4,$^)
```

`word` 是 Make 内置函数，意思是“按空格切分后取第几个元素”。

例子：

```make
$(word 2,a.o b.o c.o)
```

结果：

```text
b.o
```

Best practice：能不用第 N 个依赖就不用。大多数规则用 `$<` 或 `$^` 就够了。

如果经常要 `$(word 4,$^)`，说明规则可能太复杂，可以拆成更小的规则。

## 9. Make 函数里的空格

Make 函数格式：

```make
$(函数名 参数1,参数2)
```

推荐写法：

```make
$(word 2,$^)
```

不要写：

```make
$(word 2, $^)
```

逗号后面的空格会成为第二个参数的一部分。多数时候可能看起来没事，但容易产生奇怪空格。

也不要写：

```make
$(word 2 ,$^)
```

这里 `2 ` 可能不被当作合法数字。

Best practice：Make 函数逗号后不要随手加空格。

如果加了，Make 不一定帮你自动 trim，调试起来很烦。

## 10. `@test -f '$(FILE)'`

当前 Makefile 有：

```make
check-file:
	@test -f '$(FILE)' || { echo 'input file not found: $(FILE)'; exit 1; }
```

Make 先展开 `$(FILE)`，再把这一行交给 shell。`test -f` 检查文件是否存在；失败时 `||` 后面的命令打印错误并 `exit 1`，让 Make 停止。

开头的 `@` 是 Make 语法：执行命令，但不打印命令本身。shell 看不到这个 `@`。

Best practice：入口检查放在 `check-file` 这种 phony target，用户传错文件时早失败。

## 11. 引号

当前 Makefile 里常见：

```make
'$(FILE)'
'$@'
'$<'
```

引号是给 shell 用的，不是 Make 语法。

Make 先展开变量，然后 shell 看到带引号的路径：

```sh
nasm -f macho64 -o 'build/2021/2021.o' 'build/2021/2021.s'
```

Best practice：路径变量放进 shell 命令时加引号。

如果路径里有空格，不加引号会被 shell 拆成多个参数。

## 12. 临时环境变量

当前 Makefile 有：

```make
CARGO_TARGET_DIR='$(CARGO_TARGET_DIR)' cargo run --quiet -- '$(ABS_FILE)' > '$@'
```

这是 shell 语法，不是 Make 语法。

意思是：只对这一次 `cargo run` 设置环境变量 `CARGO_TARGET_DIR`。

展开后类似：

```sh
CARGO_TARGET_DIR='build/cargo-target' cargo run --quiet -- '/abs/path/source/2021.int' > 'build/2021/2021.s'
```

`>` 是 shell 重定向，把 `cargo run` 的标准输出写进目标文件。

Best practice：把 Cargo 的 target 目录放进 `build/cargo-target`，生成物集中管理。

如果不用 `CARGO_TARGET_DIR`，Cargo 会默认生成 `target/`，项目根目录会多一堆构建缓存。

## 13. 常见 shell 命令

```make
@mkdir -p '$(OUT_DIR)'   # 创建目录，已存在也不报错
rm -rf '$(BUILD_DIR)'    # 递归删除 build 目录
```

Best practice：`clean` 只删除明确的 build 目录。

如果 `rm -rf` 后面变量写错，可能误删不该删的东西。所以 `BUILD_DIR := build` 这种变量要简单、明确。

## 14. `.PHONY`

```make
.PHONY: help asm build run clean check-file
```

这些 target 不对应真实文件，而是命令入口。

比如：

```sh
make clean
```

即使目录里有一个叫 `clean` 的文件，Make 也会执行 `clean` 规则，而不是认为它已经完成。

Best practice：命令入口都放进 `.PHONY`。

如果不写 `.PHONY`，刚好出现同名文件时，Make 可能认为 target 已经是最新的，于是不执行命令。

## 15. 当前常用命令

```sh
make
make asm
make build
make run
make clean
```

指定其它输入文件：

```sh
make run FILE=source/foo.int
```

当前不要使用：

```sh
make run source/foo.int
```

这是把 `source/foo.int` 当 target，不是变量传参。
