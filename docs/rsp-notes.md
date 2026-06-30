# x86-64 `rsp` / `rip` / 控制流笔记

`rsp` 是栈指针寄存器，通常保存“当前栈顶”的地址。`rip` 是指令指针寄存器，保存 CPU 下一步要执行的代码地址。

这组指令的核心不是 NASM 语法，而是两个寄存器的特殊用途：

- `push` / `pop` 特殊在于它们隐式读写 `rsp`
- `jmp` / `call` / `ret` 特殊在于它们改变 `rip`
- `call` / `ret` 又把 `rip` 和 `rsp` 连在一起：返回地址放在栈上

注意区分：

```asm
rsp        ; 寄存器里的值：通常是一个栈地址
[rsp]      ; rsp 指向的内存内容：可能是地址，也可能是普通数据
```

CPU 不知道一个 64-bit 值是整数还是指针。怎么解释这些 bit，取决于你用什么指令、遵守什么调用约定。

## CPU 像解释器一样执行指令

先分清两个值：

```text
next_rip:
    当前指令按顺序执行时，下一条指令的地址
    = 当前指令地址 + 当前指令长度

final_rip:
    当前指令执行完成后，CPU 实际要执行的下一条地址
```

先用一个简化模型理解 x86 CPU：

```text
while true:
    instr_addr = rip
    instr = decode(memory[instr_addr...])
    next_rip = instr_addr + instr.length

    执行 instr

    if instr 没有改变控制流:
        final_rip = next_rip
    else:
        final_rip = instr 计算出来的新地址

    rip = final_rip
```

真实 CPU 会流水线、乱序执行、分支预测、把复杂指令拆成 micro-op；但架构语义上，可以先把它理解成这个循环。

一步指令大约经历：

```text
1. 用 rip 作为地址，取指令字节
2. 解码指令，知道 opcode、长度、操作数
3. 先算出 next_rip = rip + 指令长度
4. 读取操作数：寄存器、内存、立即数
5. 执行指令语义
6. 写回结果：寄存器、内存、flags、rip
7. 继续下一条
```

普通数据指令：

```asm
add rax, 1
```

```text
next_rip = rip + instr.length
rax = rax + 1
更新 rflags
rip = next_rip
```

普通内存指令：

```asm
mov rax, [rbx+8]
```

```text
next_rip = rip + instr.length
addr = rbx + 8
rax = memory[addr]
rip = next_rip
```

所以普通指令也会和 `rip` 交互：它们用 `rip` 取指，执行完让 `rip` 顺序前进。

## `rip` 和控制流

`rip` 决定下一条执行哪条指令。

普通指令执行完，CPU 会自动让 `rip` 指向下一条指令：

```asm
add rax, 1       ; 执行后 rip 自动变成下一条指令地址
mov rbx, rax     ; 继续执行这里
```

控制流指令会显式改变 `rip`：

```asm
jmp target       ; rip = target
call foo         ; 保存返回地址，然后 rip = foo
ret              ; 从栈里取返回地址，然后 rip = 返回地址
```

`rip` 不是普通通用寄存器，不能这样写：

```asm
mov rip, rax     ; invalid
```

要跳到 `rax` 指向的代码地址，用控制流指令：

```asm
jmp rax
call rax
```

64-bit 模式里还有常见的 RIP-relative 寻址：

```asm
lea rax, [rel label]   ; rax = label 的地址
mov eax, [rel value]   ; eax = value 处的内容
```

这不是把 `rip` 当普通寄存器用，而是机器码支持“相对当前指令地址”的寻址形式。

RIP-relative 的意思是：机器码里不直接保存完整绝对地址，而是保存相对当前指令附近的偏移。

```text
effective_address = next_rip + displacement
```

这里的 `next_rip` 是当前指令结束后的地址。这样程序被加载到不同内存地址时，代码仍然能靠“当前位置 + 偏移”找到附近的数据，常用于 position-independent code。

NASM 里的 `rel` 就是 relative。它不是指令，也不是寄存器，而是告诉汇编器：

```asm
[rel label]
```

把 `label` 编码成相对 `rip` 的偏移，而不是绝对地址。

汇编器/链接器根据当前位置和 `label` 位置算：

```text
displacement = label - next_rip
```

CPU 执行时再算：

```text
effective_address = next_rip + displacement
```

比如：

```text
next_rip = 0x1000
label    = 0x0ff0
displacement = label - next_rip = -0x10
effective_address = 0x1000 + (-0x10) = 0x0ff0
```

所以可以直观理解成：`rel` 根据当前指令位置和 `label` 位置，用相对偏移访问到那个 `label`。它不改变 `rip`，只是用 `rip`/`next_rip` 来算地址。

这里的 `next_rip` 一定是“当前指令按顺序排列时的下一条指令地址”，不是跳转指令最终跳到的 target。

```asm
1000: lea rax, [rel msg]     ; 假设长度是 7
1007: jmp somewhere
100c: msg:
```

这条 `lea` 里的 `rel msg` 用的是：

```text
next_rip = 0x1007
displacement = msg - 0x1007
```

它不关心后面的 `jmp somewhere` 最终跳到哪里。

为什么要用 `rel`，而不是把绝对地址写进机器码？

因为现代程序通常不想依赖“数据一定在某个固定地址”。如果用相对地址，代码和数据整体搬到别的位置时，它们之间的距离不变，访问仍然正确。

```text
原来:
next_rip = 0x1000
msg      = 0x1020
偏移     = +0x20

搬家后:
next_rip = 0x5000
msg      = 0x5020
偏移     = 仍然 +0x20
```

但如果机器码里写死绝对地址：

```text
msg = 0x1020
```

程序搬到 `0x5000` 后，真正的 `msg` 在 `0x5020`，机器码却还去访问 `0x1020`，就错了。除非 loader 启动时把机器码里的绝对地址修一遍。

用 `rel` 的好处：

- 代码更容易重定位：加载到不同地址也能工作
- 适合 ASLR：操作系统可以随机化程序加载地址
- 共享库更容易共享代码页：不用为每个进程改写代码里的绝对地址
- 指令通常更短：常用 32-bit displacement，而不是完整 64-bit 地址

所以 `rel` 更像：

```text
从当前这条指令附近找 msg
```

绝对地址更像：

```text
去固定门牌号 0x0000000000401020 找 msg
```

绝对地址不是不能用，只是更依赖加载位置，或者需要 loader 做 relocation。

`lea` 是 Load Effective Address：计算地址，把地址值放进寄存器。它不读内存。

它常出现的原因是：`mov` 不能直接把寄存器表达式当普通值搬到目标寄存器。

```asm
mov rax, rbx+rcx*4+16    ; invalid
```

`mov` 的源操作数只能是寄存器、内存或立即数。`rbx+rcx*4+16` 这种表达式只被 x86 支持在“内存寻址格式”里。

如果不用 `lea`，要写成多条算术指令：

```asm
mov rax, rcx
shl rax, 2
add rax, rbx
add rax, 16
```

用 `lea` 可以借用内存寻址格式，一条指令算出同一个值：

```asm
lea rax, [rbx+rcx*4+16]  ; rax = rbx + rcx*4 + 16
```

```asm
lea rax, [rbx+8]       ; rax = rbx + 8
mov rax, [rbx+8]       ; rax = memory[rbx + 8]
```

所以：

```asm
lea rax, [rel label]   ; rax = label 的地址
mov rax, [rel label]   ; rax = label 地址处的 8 字节内容
```

`lea` 看起来像内存寻址，但实际只算括号里的地址表达式，不访问那块内存。

可以把 `lea` 理解成：

```text
dst = address_expression
```

而不是：

```text
dst = memory[address_expression]
```

它能算的不是任意四则运算，只是 x86 地址表达式：

```asm
lea dst, [base + index*scale + displacement]
```

其中 `scale` 只能是 `1`、`2`、`4`、`8`。

```asm
lea rax, [rbx+rcx]        ; rax = rbx + rcx
lea rax, [rbx+rcx*4+16]   ; rax = rbx + rcx*4 + 16
lea rax, [rdi+rdi*2]      ; rax = rdi * 3
```

不能做这些：

```asm
lea rax, [rbx*3]          ; invalid，scale 不能是 3
lea rax, [rbx-rcx]        ; invalid，没有寄存器减寄存器
lea rax, [rbx/2]          ; invalid
lea rax, [rbx*16]         ; invalid，scale 不能是 16
```

另外，`lea` 不修改 flags，所以也常被拿来做不影响条件码的小算术。

## 控制流指令特殊在哪里

普通数据指令主要改寄存器或内存：

```asm
add rax, 1       ; 改 rax
mov [rsp], rax   ; 改内存
```

控制流指令改的是“下一步执行哪里”，也就是改 `rip`：

```text
普通指令:
    final_rip = next_rip

RIP-relative 寻址:
    effective_address = next_rip + displacement
    final_rip = next_rip

相对 jmp:
    target = next_rip + displacement
    final_rip = target

相对 jcc:
    target = next_rip + displacement
    if 条件成立:
        final_rip = target
    else:
        final_rip = next_rip

相对 call:
    target = next_rip + displacement
    rsp -= 8
    [rsp] = next_rip
    final_rip = target

ret:
    final_rip = [rsp]
    rsp += 8
```

所以 `next_rip` 是“基准地址”，`final_rip` 是“执行结果”。`rel` / RIP-relative 和相对跳转都常用 `next_rip` 做基准；只有控制流指令会把 `final_rip` 改成别的地方。

x86 通常不叫 `br`，而是叫：

```asm
jmp     ; unconditional branch
jcc     ; conditional branch，比如 je/jne/jg
call
ret
```

相对跳转例子：

```asm
1000: eb 05        jmp short target
1002: ...
1007: target:
```

`jmp short` 长度是 2：

```text
next_rip = 0x1002
displacement = 0x05
target = 0x1002 + 0x05 = 0x1007
final_rip = 0x1007
```

所以 `call` / `ret` 最值得注意：它们把控制流信息放进了普通内存栈里。

```text
[rsp] 可能只是普通数据
[rsp] 也可能是 ret 将要跳转到的代码地址
```

这也是为什么乱改 `rsp` 或覆盖返回地址会直接破坏控制流。

## 栈顶和分配

x86-64 的栈通常向低地址增长。

```asm
sub rsp, 32     ; 给当前函数分配 32 字节栈空间
add rsp, 32     ; 释放这 32 字节
```

进入函数时，操作系统/运行时已经给线程准备了一段栈内存；但当前函数自己的局部栈空间仍然要靠 `sub rsp, n` 或 `push` 来占用。

不要把还没占用的空间当成自己的：

```asm
mov [rsp-8], rax    ; 不严谨：这 8 字节还没通过 rsp 分配给你
```

要写新栈槽，先移动 `rsp`：

```asm
push rax
; 等价语义：
; rsp -= 8
; [rsp] = rax
```

## `push` / `pop`

64-bit 模式下普通 `push` / `pop` 一般按 8 字节操作：

```asm
push rax
; rsp -= 8
; [rsp] = rax

pop rax
; rax = [rsp]
; rsp += 8
```

所以：

- `push` 先让 `rsp` 变小，再写 `[rsp]`
- `pop` 先读 `[rsp]`，再让 `rsp` 变大

## `call` / `ret`：控制流和栈的连接点

普通 near `call` 可以这样理解：

```asm
call foo
; push next_rip
; rip = foo
```

也就是：

```asm
rsp -= 8
[rsp] = 返回地址
rip = foo
```

所以在 `foo` 刚开始时：

```asm
rsp      ; 栈顶地址
[rsp]    ; 返回地址，也就是 call 后面那条指令的地址
```

普通 `ret` 可以这样理解：

```asm
ret
; rip = [rsp]
; rsp += 8
```

也就是从当前栈顶取出返回地址，跳回调用点之后继续执行。

所以 `call` / `ret` 的特殊性是：

```text
call: 改 rip，并把旧控制流的返回位置存到 [rsp]
ret:  从 [rsp] 取出返回位置，并写回 rip
```

这就是函数调用能“跳出去再回来”的最小机制。

## 它们不是 NASM 语法糖

`push`、`pop`、`call`、`ret` 不是 NASM 展开的宏，也不是汇编层面的语法糖。它们是真实机器指令，有自己的 opcode。

上面的写法只是语义模型，方便理解它们对 `rsp`、内存、`rip` 的影响。

## 寄存器里的值没有类型

除了特殊用途寄存器有强约定，通用寄存器本身没有“类型”：

```asm
add rax, 5       ; 把 rax 当整数
mov rax, [rdi]   ; 把 rdi 当地址
```

常见理解：

```text
rsp   通常是栈地址
[rsp] 当前栈顶内容，可能是地址，也可能是普通数据
rip   下一条要执行的代码地址，由普通执行自动前进，或被控制流指令改写
rax/rbx/rcx/... 只是 64-bit 值，可能当数据，也可能当地址
```

## 小限制

`rsp` 可以作为内存寻址的 base：

```asm
mov rax, [rsp+8]
```

但不能作为 scaled index：

```asm
mov rax, [rbx + rsp*2]   ; invalid
```

这是 x86 地址编码限制，不是 NASM 的特殊规则。

## 图

见 [rsp_push_pop_call_ret.svg](rsp_push_pop_call_ret.svg)。
