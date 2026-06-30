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

## 2. Make 和 shell 的分工

Makefile 里有两层东西：

```text
Make 负责：
- 展开变量：$(FILE)、$(ASM)
- 展开函数：$(notdir ...)、$(basename ...)
- 判断目标和依赖是否需要重建

shell 负责：
- 真正执行命令：cargo、nasm、ar、rustc、test、mkdir、rm
- 处理重定向：>
- 处理 shell 运算符：||、{ ...; }
```

例子：

```make
$(OBJ): $(ASM)
	nasm -f $(NASM_FORMAT) -o '$@' '$<'
```

Make 先把变量展开成类似：

```sh
nasm -f macho64 -o 'build/2021/2021.o' 'build/2021/2021.s'
```

然后这一行才交给 shell 执行。

## 3. `$` 在 Makefile 里的作用

在 Makefile 里，`$` 通常表示“让 Make 展开某个东西”。

常见形式：

```make
$(FILE)      # 展开 Make 变量 FILE
$(notdir x)  # 调用 Make 内置函数 notdir
$@           # Make 自动变量：当前目标
$<           # Make 自动变量：第一个依赖
$^           # Make 自动变量：所有依赖
```

所以：

```make
$(BUILD_DIR)
```

不是 shell 命令，而是让 Make 读取变量 `BUILD_DIR`。

带括号和不带括号有区别：

```make
$(FILE)  # 多字符 Make 变量 FILE
$F       # 单字符 Make 变量 F
```

不要把多字符变量写成：

```make
$FILE
```

Make 会把它理解成：

```text
$F + 普通文本 ILE
```

不是变量 `FILE`。

例子：

```make
FILE := source/2021.int

bad:
	@echo $FILE

good:
	@echo $(FILE)
```

`bad` 可能输出：

```text
ILE
```

`good` 才会输出：

```text
source/2021.int
```

但是 Make 自动变量本身就是单字符，所以通常不带括号：

```make
$@
$<
$^
$+
```

如果你想把 `$` 留给 shell，用两个 `$`：

```make
print-home:
	@echo $$HOME
```

这里 `$$` 是一个整体，意思是“请 Make 输出一个真正的 `$`”。
`$$` 本身就是转义，不用括号。

Make 会先把：

```make
$$HOME
```

变成 shell 看到的：

```sh
$HOME
```

然后 shell 再展开 `$HOME`，比如：

```text
/Users/zhangyongzhuo
```

如果写成：

```make
	@echo $HOME
```

Make 会先尝试展开 `$H`，shell 收不到正确的 `$HOME`。

也不要写成：

```make
	@echo $$(HOME)
```

Make 会先把 `$$` 变成 `$`，shell 看到的是：

```sh
echo $(HOME)
```

在 shell 里 `$(...)` 是命令替换，它会尝试执行名为 `HOME` 的命令，这通常是错的。

Best practice：Make 变量用 `$(VAR)`，shell 变量在 recipe 里写成 `$$VAR`。

如果在 recipe 里写 `$HOME`，Make 会先尝试展开 `$H`，结果通常不是你想要的。

可以简单记：

```make
$(FILE)   # 给 Make 用：多字符变量
$@        # 给 Make 用：自动变量，当前目标
$<        # 给 Make 用：自动变量，第一个依赖
$$HOME    # 留给 shell 用，最终 shell 看到 $HOME
```

## 4. 当前构建链路

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

## 5. Make 变量赋值

例子：

```make
BUILD_DIR := build
RUST_TARGET := x86_64-apple-darwin
```

`:=` 是立即赋值。右边会在这一行就展开。

```make
FILE ?= source/2021.int
```

`?=` 是默认赋值。只有 `FILE` 没有被设置时才生效。

使用变量：

```make
$(BUILD_DIR)
$(FILE)
```

Best practice：

- 固定配置用变量，比如 `RUST_TARGET`、`NASM_FORMAT`。
- 可被命令行覆盖的输入用 `?=`，比如 `FILE ?= source/2021.int`。

如果到处硬编码路径，以后改文件名或 target 会改很多地方。

## 6. 从输入文件名生成输出名

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

## 7. 绝对路径

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

## 8. 规则、目标和依赖

Make 规则长这样：

```make
目标: 依赖1 依赖2
	命令
```

注意：命令行前面必须是 tab，不是普通空格。

例子：

```make
$(OBJ): $(ASM)
	nasm -f $(NASM_FORMAT) -o '$@' '$<'
```

含义：

```text
要生成 $(OBJ)，需要先有 $(ASM)。
如果 $(ASM) 比 $(OBJ) 新，或者 $(OBJ) 不存在，就运行下面的命令。
```

Best practice：把真实文件写成依赖，让 Make 自己判断什么时候重建。

如果所有东西都写在一个 `run` 命令里，Make 就不知道哪些文件变了，也不能做增量构建。

## 9. 自动变量

Make 规则里的常用自动变量：

```make
$@   # 当前目标
$<   # 第一个依赖
$^   # 所有依赖，去重
$+   # 所有依赖，不去重，保留顺序
```

例子：

```make
$(OBJ): $(ASM)
	nasm -f $(NASM_FORMAT) -o '$@' '$<'
```

如果展开后是：

```make
build/2021/2021.o: build/2021/2021.s
```

那么：

```text
$@ = build/2021/2021.o
$< = build/2021/2021.s
```

命令等价于：

```sh
nasm -f macho64 -o 'build/2021/2021.o' 'build/2021/2021.s'
```

如果有多个依赖：

```make
target: a.o b.o c.o
```

那么：

```text
$< = a.o
$^ = a.o b.o c.o
```

Best practice：在规则里优先用 `$@`、`$<`、`$^`，不要重复手写目标文件名和依赖文件名。

如果手写路径，规则名改了，命令里的路径可能忘记同步。

## 10. 第 N 个依赖

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

## 11. Make 函数里的空格

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

## 12. `@test -f '$(FILE)'`

当前 Makefile 有：

```make
check-file:
	@test -f '$(FILE)' || { echo 'input file not found: $(FILE)'; exit 1; }
```

这一行是 shell 命令，不是 Make 函数。

Make 会先展开：

```make
$(FILE)
```

如果 `FILE=source/2021.int`，shell 实际看到的是：

```sh
test -f 'source/2021.int' || { echo 'input file not found: source/2021.int'; exit 1; }
```

逐段解释：

```sh
test -f 'source/2021.int'
```

判断这个路径是不是一个存在的普通文件。

```sh
||
```

表示“如果前面的命令失败，就执行后面的命令”。

```sh
{ echo 'input file not found: source/2021.int'; exit 1; }
```

这是 shell 命令组：

- `echo ...` 打印错误信息。
- `exit 1` 用失败状态退出，让 Make 停止。
- `}` 前面的分号必须有。

最前面的 `@`：

```make
	@test ...
```

这是 Make 的语法，不是 shell 的语法。

它表示：执行这条命令，但不要把命令本身打印出来，只打印命令的输出。

Make 会先把行首的 `@` 去掉，再把剩下的命令交给 shell：

```sh
test -f 'source/2021.int'
```

所以 shell 实际看不到这个 `@`。

如果不写 `@`，运行时会看到：

```text
test -f 'source/2021.int' || { echo ...; exit 1; }
```

这对调试有用，但平时输出比较吵。

Best practice：入口检查可以用 `check-file` 这种 phony target；用户传错文件时要早失败，并给清楚错误。

如果不检查，后面 `cargo run` 可能报一个更绕的文件读取错误。

## 13. 引号

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

## 14. 临时环境变量

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

## 15. 常见 shell 命令

```make
@mkdir -p '$(OUT_DIR)'
```

`mkdir -p` 的意思：

- 目录不存在就创建。
- 父目录不存在也一起创建。
- 目录已存在也不报错。

```make
rm -rf '$(BUILD_DIR)'
```

`rm -rf` 的意思：

- `-r`：递归删除目录。
- `-f`：不存在也不报错。

Best practice：`clean` 只删除明确的 build 目录。

如果 `rm -rf` 后面变量写错，可能误删不该删的东西。所以 `BUILD_DIR := build` 这种变量要简单、明确。

## 16. `.PHONY`

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

## 17. 当前常用命令

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

## 18. 简短 best-practice 总结

| 写法 | 为什么好 | 不这样的问题 |
| --- | --- | --- |
| `make run FILE=...` | 正常 Make 变量传参 | `make run file` 会把 file 当 target |
| `FILE ?= ...` | 提供默认输入，也允许覆盖 | 每次都要写完整命令 |
| `BUILD_DIR := build` | 生成物集中 | 根目录污染 git |
| `CARGO_TARGET_DIR := build/cargo-target` | Cargo 缓存也集中 | 生成默认 `target/` |
| `$(basename $(notdir ...))` | 从输入自动推导输出名 | 换输入文件时要改 Makefile |
| `$(abspath ...)` | 给工具稳定路径 | 工作目录变化时可能找不到文件 |
| `$@` / `$<` / `$^` | 避免重复写路径 | 目标名改了命令忘改 |
| `@test -f ...` | 输入不存在时早失败 | 后面报错更绕 |
| 路径加引号 | 防止空格拆参数 | 有空格路径会炸 |
| `.PHONY` | 命令入口稳定执行 | 同名文件会让 target 跳过 |
