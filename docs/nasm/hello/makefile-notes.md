# NASM Hello 的 Makefile 笔记

整理时间：2026-07-01

范围：这里只讨论 `Nasm/Hello/Makefile`。手动执行 `nasm` / `ld` 的命令和 `xcrun` 背景见 [nasm-macos-hello-notes.md](nasm-macos-hello-notes.md)。

## 1. 最终 Makefile

```make
PROGRAM ?= hello
SDK_VERSION := $(shell xcrun --show-sdk-version)

.DEFAULT_GOAL := run

.PHONY: run build assemble clean

run: build/$(PROGRAM)
	./build/$(PROGRAM)

build: build/$(PROGRAM)

assemble: build/$(PROGRAM).o

build/$(PROGRAM): build/$(PROGRAM).o
	/usr/bin/ld -static -arch x86_64 -platform_version macos 11.0 $(SDK_VERSION) -e start $< -o $@

build/$(PROGRAM).o: $(PROGRAM).asm
	mkdir -p $(@D)
	nasm -f macho64 $< -o $@

clean:
	rm -rf build
```

依赖链：

```text
run -> build/hello -> build/hello.o -> hello.asm
```

常用命令：

```sh
make              # 默认执行 run
make build        # 只构建 build/hello
make assemble     # 只构建 build/hello.o
make clean        # 删除 build/
make PROGRAM=foo  # 使用 foo.asm，输出 build/foo.o 和 build/foo
```

## 2. 为什么 `$(xcrun ...)` 在 Makefile 里会失败

终端里这条命令可以工作：

```sh
/usr/bin/ld -static -arch x86_64 -platform_version macos 11.0 $(xcrun --show-sdk-version) -e start build/hello.o -o build/hello
```

因为在 shell 里：

```sh
$(xcrun --show-sdk-version)
```

是 command substitution：shell 先执行 `xcrun --show-sdk-version`，再把输出的 SDK 版本填回命令。

但 recipe 里的命令不是直接交给 shell。`make` 会先展开 Makefile 语法，再把展开后的命令交给 shell。所以 recipe 里的：

```make
$(xcrun --show-sdk-version)
```

会先被 `make` 当成自己的 `$(...)` 展开，而不是 shell 的 command substitution。这里没有 `xcrun` 这个 make 函数，通常就会按变量引用处理并展开为空。

结果 shell 实际拿到的是：

```sh
/usr/bin/ld -static -arch x86_64 -platform_version macos 11.0 -e start build/hello.o -o build/hello
```

`-platform_version` 需要三个参数：

```text
-platform_version macos 11.0 <sdk-version>
```

SDK 版本消失后，`ld` 把后面的 `-e` 当成版本号解析，所以报：

```text
ld: -platform_version: malformed version number '-e' cannot fit in 32-bit xxxx.yy.zz
```

## 3. 两种正确写法

推荐写法：

```make
SDK_VERSION := $(shell xcrun --show-sdk-version)
```

然后 recipe 里使用：

```make
$(SDK_VERSION)
```

`$(shell ...)` 是 make 的内置函数，意思是：让 `make` 调用 shell 执行命令，并把 stdout 作为函数结果。这里配合 `:=`，命令会在变量赋值时执行一次。

注意：这里执行命令的 shell 是 make 选择的 shell，通常是 `/bin/sh`，不是你当前交互终端里的 zsh。除非 Makefile 里显式设置：

```make
SHELL := /bin/zsh
```

另一种写法是把 shell command substitution 留到 recipe 执行阶段：

```make
	/usr/bin/ld ... $$(xcrun --show-sdk-version) ...
```

`$$` 会先被 `make` 变成单个 `$`，所以 shell 最后看到：

```sh
$(xcrun --show-sdk-version)
```

这也能工作，但这里 SDK 版本是 Makefile 里的构建配置，用 `SDK_VERSION := $(shell ...)` 更清楚。

## 4. Phony target 和真实产物

构建类 phony target 最好依赖一个真实文件：

```make
run: build/$(PROGRAM)
build: build/$(PROGRAM)
assemble: build/$(PROGRAM).o
```

真实文件负责时间戳判断：

```make
build/$(PROGRAM): build/$(PROGRAM).o
build/$(PROGRAM).o: $(PROGRAM).asm
```

这样 `hello.asm` 没变时，`make` 不会重新执行 `nasm` / `ld`。

不要把 target 名和实际输出写成两个路径：

```make
hello.o: hello.asm
	nasm -f macho64 hello.asm -o build/hello.o
```

这里 target 是 `hello.o`，但实际生成的是 `build/hello.o`。因为 `hello.o` 永远不存在，make 会认为它永远过期。

`clean` 这种 target 不产出文件，它就是动作本身，所以直接写 recipe：

```make
.PHONY: clean

clean:
	rm -rf build
```

## 5. `PROGRAM ?= hello`

`PROGRAM` 是当前 Makefile 的可配置变量：

```make
PROGRAM ?= hello
```

含义：如果外部没有指定 `PROGRAM`，默认用 `hello`。

```sh
make              # PROGRAM=hello
make PROGRAM=foo  # PROGRAM=foo
```

Make 变量名不强制大写；大写只是常见约定，表示“这是构建配置，可以从命令行覆盖”。

## 6. 自动变量和 `$(@D)`

Make 执行规则时会自动设置一些 automatic variables：

```make
$@  # 当前 target
$<  # 第一个 prerequisite
$^  # 所有 prerequisites，去重
$+  # 所有 prerequisites，保留重复项
$?  # 比 target 更新的 prerequisites
$*  # pattern rule 中 % 匹配到的 stem
$%  # archive member target 的 member 名
```

在当前规则里：

```make
build/$(PROGRAM).o: $(PROGRAM).asm
	mkdir -p $(@D)
	nasm -f macho64 $< -o $@
```

如果 `PROGRAM=hello`，展开关系是：

```text
$@     -> build/hello.o
$<     -> hello.asm
$(@D)  -> build
```

所以 recipe 近似变成：

```sh
mkdir -p build
nasm -f macho64 hello.asm -o build/hello.o
```

`$@` 是 automatic variable，可以写成 `$(@)`；两者等价。

## 7. `D` / `F` 后缀

`D` / `F` 是 automatic variables 的特殊变体，只作用在 automatic variables 上：

```make
$(@D)  # $@ 的目录部分
$(@F)  # $@ 的文件名部分
$(<D)  # $< 的目录部分
$(<F)  # $< 的文件名部分
$(^D)  # $^ 中每一项的目录部分
$(^F)  # $^ 中每一项的文件名部分
```

完整地说，`D` / `F` 可以作用在这些 automatic variables 上：

```text
$@  $%  $<  $?  $^  $+  $*
```

也就是：

```text
$(@D)  $(@F)
$(%D)  $(%F)
$(<D)  $(<F)
$(?D)  $(?F)
$(^D)  $(^F)
$(+D)  $(+F)
$(*D)  $(*F)
```

普通变量没有这种“最后一个字母自动变目录”的规则：

```make
FOO := build/hello.o
FOOD := abc

$(FOOD)  # 变量 FOOD，不是 FOO + D
```

普通变量要用路径函数：

```make
$(dir $(FOO))       # build/
$(notdir $(FOO))    # hello.o
$(basename $(FOO))  # build/hello
$(suffix $(FOO))    # .o
```

`$(@D)` 和 `$(dir $@)` 不完全一样：

```make
$(@D)      # build
$(dir $@)  # build/
```

类似 `D` / `F` 的 automatic variable 后缀只有这两个。其他路径处理用 make 函数。

## 8. Make 语法和 shell 语法不同

核心区别：

```text
Make:  $(...) 是 make 的展开语法，可能是变量引用，也可能是 make 函数调用。
Shell: $name / ${name} 是变量展开；$(...) 是 command substitution。
```

Make 里的例子：

```make
$(PROGRAM)       # make 变量
$(SDK_VERSION)   # make 变量
$(dir $@)        # make 函数
$(shell date)    # make 函数：让 make 调 shell 执行 date
```

Shell 里的例子：

```sh
$PROGRAM         # shell 变量
${PROGRAM}       # shell 变量，边界更清楚
$(date)          # shell command substitution，执行 date
```

所以不能只看 `$` 和括号的长相；要看当前是谁在解释这段文本。

在 Makefile recipe 里有两轮解释：

```make
show:
	echo $(PROGRAM)
	echo $$PROGRAM
	echo $$(date)
```

第一行是 make 变量，第二行留给 shell 展开变量，第三行留给 shell 执行命令。`make` 交给 shell 前大致会变成：

```sh
echo hello
echo $PROGRAM
echo $(date)
```

Shell 靠语法区分变量和命令：

```sh
FOO=hello  # variable assignment
$FOO       # variable expansion
${FOO}     # variable expansion
FOO        # 在命令位置时是 command name
$(FOO)     # command substitution，执行 FOO
```

`$FOO` / `${FOO}` 查当前 shell 的变量表；这个表包括当前 shell 自己定义的变量和从父进程继承来的 environment variables。当前 shell 自己展开变量时不需要 `export`，只有想让子进程也看到变量时才需要 `export`。

## 9. `make` 命令参数

`make` 命令后面的内容主要分三类：

```sh
make [options] [targets] [VARIABLE=value]
```

target 决定做什么：

```sh
make
make run
make build
make assemble
make clean
```

`VARIABLE=value` 覆盖 Makefile 变量：

```sh
make PROGRAM=foo
make SDK_VERSION=15.5
```

options 控制 make 自己怎么运行：

```sh
make -n build              # dry run，只打印，不执行
make -B run                # 强制重建
make -j4                   # 最多并行 4 个任务
make -C Nasm/Hello         # 进入目录后执行
make -f MyMakefile         # 使用指定 Makefile
make -s                    # 少打印 recipe 命令本身
```

这些短选项对应的长选项和记忆方式：

| 短选项 | 长选项 | 记法 |
| --- | --- | --- |
| `-n` | `--dry-run` / `--just-print` | no execute，只打印不执行 |
| `-B` | `--always-make` | build anyway，强制重建 |
| `-j4` | `--jobs=4` | jobs，并行任务数 |
| `-C DIR` | `--directory=DIR` | change directory，先进入目录 |
| `-f FILE` | `--file=FILE` / `--makefile=FILE` | file，指定 Makefile |
| `-s` | `--silent` / `--quiet` | silent，少打印命令本身 |

也可以写成长选项：

```sh
make --dry-run build
make --always-make run
make --jobs=4
make --directory=Nasm/Hello
make --file=MyMakefile
make --silent
```

注意：

```sh
make run build
```

不是给 `run` 传 `build` 参数，而是执行两个 target：`run` 和 `build`。给 Makefile 传配置，优先用：

```sh
make run PROGRAM=hello
```
