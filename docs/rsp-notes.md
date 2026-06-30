# x86-64 `rsp` / `rip` / 控制流笔记

`rsp` 是栈指针寄存器，通常保存当前栈顶地址。栈顶就是 last-in-first-out 里的“最后放入、最先取出”的位置。
`rip` 是指令指针寄存器，决定 CPU 下一步从哪里取指令。

x86-64 的栈通常向低地址增长，所以 `push` 会让 `rsp` 变小，`pop` 会让 `rsp` 变大。

核心关系：

- 普通指令主要改寄存器、内存或 flags，`rip` 顺序前进
- `jmp` / `jcc` / `call` / `ret` 会改变控制流，也就是改变最终的 `rip`
- `push` / `pop` 隐式读写 `rsp`
- `call` / `ret` 把 `rip` 和 `rsp` 连起来：返回地址放在栈上

寄存器里的值没有类型。CPU 不知道一个 64-bit 值是整数还是指针，怎么解释取决于指令和调用约定。

```asm
rsp        ; 寄存器里的值：通常是一个栈地址
[rsp]      ; rsp 指向的内存内容：可能是地址，也可能是普通数据
```

## 指令执行模型

先分清两个值：

```text
next_rip:
    当前指令按顺序执行时，下一条指令的地址
    = 当前指令地址 + 当前指令长度

final_rip:
    当前指令执行完成后，CPU 实际要执行的下一条地址
```

可以把 x86 CPU 的架构语义先理解成一个硬件解释器：

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

真实 CPU 会流水线、乱序执行、分支预测、把复杂指令拆成 micro-op；这里讲的是架构语义。

一条普通指令大约经历：

```text
1. 用 rip 作为地址，取指令字节
2. 解码指令，知道 opcode、长度、操作数
3. 算出 next_rip = 当前指令地址 + 指令长度
4. 读取操作数：寄存器、内存、立即数
5. 执行指令语义
6. 写回结果：寄存器、内存、flags、rip
```

普通数据指令：

```asm
add rax, 1
```

```text
next_rip = rip + instr.length
rax = rax + 1
更新 rflags
final_rip = next_rip
```

普通内存指令：

```asm
mov rax, [rbx+8]
```

```text
next_rip = rip + instr.length
addr = rbx + 8
rax = memory[addr]
final_rip = next_rip
```

所以普通指令也会和 `rip` 交互：它们用 `rip` 取指，执行完让 `rip` 顺序前进。

## 控制流

`rip` 不是普通通用寄存器，不能这样写：

```asm
mov rip, rax     ; invalid
```

要改变执行位置，用控制流指令：

```asm
jmp rax
call rax
ret
```

x86 通常不叫 `br`，而是叫：

```asm
jmp     ; unconditional branch
jcc     ; conditional branch，比如 je/jne/jg
call
ret
```

常见控制流语义：

```text
普通指令:
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

所以：

```text
next_rip 是基准地址
final_rip 是执行结果
```

## RIP-relative 和 `rel`

64-bit 模式常见 RIP-relative 寻址：

```asm
lea rax, [rel label]   ; rax = label 的地址
mov eax, [rel value]   ; eax = value 处的内容
```

它不是跳转，也不改变 `rip`。它只是用 `next_rip` 参与地址计算：

```text
effective_address = next_rip + displacement
final_rip = next_rip
```

`displacement` 是机器码里编码的有符号偏移。对 `[rel label]`：

```text
displacement = label_address - next_rip
```

如果 `label` 在同一个汇编文件里，汇编器通常能算出这个偏移；如果是外部符号，汇编器先留下 relocation，链接器再填。

`rel` 是 NASM 的提示，意思是 relative：

```asm
[rel label]
```

它告诉汇编器：把 `label` 编码成相对 `rip` 的偏移，而不是绝对地址。

数字例子：

```text
next_rip = 0x1000
label    = 0x0ff0
displacement = label - next_rip = -0x10
effective_address = 0x1000 + (-0x10) = 0x0ff0
```

注意：这里的 `next_rip` 一定是当前指令按顺序排列时的下一条指令地址，不是 jump 最终跳到的 target。

```asm
1000: lea rax, [rel msg]     ; 假设长度是 7
1007: jmp somewhere
100c: msg:
```

这条 `lea` 用的是：

```text
next_rip = 0x1007
displacement = msg - 0x1007
```

它不关心后面的 `jmp somewhere` 跳到哪里。

为什么用 `rel`，而不是把绝对地址写进机器码？

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

代码和数据整体搬家后，相对偏移不变；如果机器码里写死 `msg = 0x1020`，搬到 `0x5000` 后就会访问旧地址，除非 loader 再做 relocation。

`rel` 的好处：

- 代码更容易重定位
- 适合 ASLR
- 共享库更容易共享代码页
- 常用 32-bit displacement，比完整 64-bit 地址短

## `lea`

`lea` = Load Effective Address。它计算地址表达式，把结果放进寄存器，不读内存。

```asm
lea rax, [rbx+8]       ; rax = rbx + 8
mov rax, [rbx+8]       ; rax = memory[rbx + 8]
```

`mov` 不能直接把寄存器表达式当普通值搬到目标寄存器：

```asm
mov rax, rbx+rcx*4+16    ; invalid
```

如果不用 `lea`，要写成多条算术指令：

```asm
mov rax, rcx
shl rax, 2
add rax, rbx
add rax, 16
```

用 `lea` 可以借用内存寻址格式算同一个值：

```asm
lea rax, [rbx+rcx*4+16]  ; rax = rbx + rcx*4 + 16
```

取地址和读内容的区别：

```asm
lea rax, [rel label]   ; rax = label 的地址
mov rax, [rel label]   ; rax = label 地址处的 8 字节内容
```

可以把 `lea` 理解成：

```text
dst = address_expression
```

不是：

```text
dst = memory[address_expression]
```

`lea` 能算的是 x86 地址表达式，不是任意四则运算：

```asm
lea dst, [base + index*scale + displacement]
```

`scale` 只能是 `1`、`2`、`4`、`8`。

可以：

```asm
lea rax, [rbx+rcx]        ; rax = rbx + rcx
lea rax, [rbx+rcx*4+16]   ; rax = rbx + rcx*4 + 16
lea rax, [rdi+rdi*2]      ; rax = rdi * 3
```

不可以：

```asm
lea rax, [rbx*3]          ; invalid，scale 不能是 3
lea rax, [rbx-rcx]        ; invalid，没有寄存器减寄存器
lea rax, [rbx/2]          ; invalid
lea rax, [rbx*16]         ; invalid，scale 不能是 16
```

另外，`lea` 不修改 flags，所以也常被用来做不影响条件码的小算术。

## `rsp` 和栈

x86-64 的栈通常向低地址增长。

```text
高地址
  ^
  |
  |  [旧的栈内容]
  |  [调用者数据]
  |
rsp -> [当前栈顶]      ; 最后 push 的值，下一次 pop 会取这里
  |
  |  [还没分配给当前函数的空间]
  v
低地址

push x:
    rsp 往低地址移动，然后写 [rsp]

pop x:
    读 [rsp]，然后 rsp 往高地址移动
```

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

`pop` 反过来：

```asm
pop rax
; rax = [rsp]
; rsp += 8
```

所以：

- `push` 先让 `rsp` 变小，再写 `[rsp]`
- `pop` 先读 `[rsp]`，再让 `rsp` 变大

`call` 的栈效果像 `push next_rip`：

```asm
call foo
; rsp -= 8
; [rsp] = next_rip
; final_rip = foo
```

所以在 `foo` 刚开始时：

```asm
rsp      ; 栈顶地址
[rsp]    ; 返回地址，也就是 call 后面那条指令的地址
```

`ret` 从栈顶取返回地址：

```asm
ret
; final_rip = [rsp]
; rsp += 8
```

`call` / `ret` 的特殊性：

```text
call: 改 rip，并把旧控制流的返回位置存到 [rsp]
ret:  从 [rsp] 取出返回位置，并写回 rip
```

这就是函数调用能跳出去再回来的最小机制。乱改 `rsp` 或覆盖返回地址会直接破坏控制流。

`push`、`pop`、`call`、`ret` 不是 NASM 展开的宏，也不是汇编层面的语法糖。它们是真实机器指令，有自己的 opcode；上面的写法只是语义模型。

## 其他小点

通用寄存器本身没有类型：

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
