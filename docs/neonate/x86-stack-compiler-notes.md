# x86-64 Stack 和简单编译策略笔记

整理时间：2026-07-02

这份笔记总结 x86-64 里 memory、stack、`rsp`、`rax`，以及教学版编译器为什么会生成一些重复指令。

## 1. 内存不是 64-bit 数组

x86-64 的内存是 byte-addressable，可以先理解成一个 byte array：

```text
address 0x1000 -> 1 byte
address 0x1001 -> 1 byte
address 0x1002 -> 1 byte
...
```

`64-bit` 主要指寄存器、指针、返回地址等常见机器值的宽度：

```text
64 bits = 8 bytes
```

所以内存的基本地址单位是 byte，不是 64-bit word。CPU 可以按不同宽度访问内存，比如 1、2、4、8 bytes。

## 2. 为什么图里常用 8-byte slot

`stack slot` 指编译器在当前函数的栈区域里划出来的一小块固定位置，用来存一个值。它不是硬件概念；CPU 只知道地址和 bytes，`slot` 是编译器和人为了管理栈空间起的名字。

在 x86-64 里，很多栈上的东西刚好是 8 bytes：

- 指针是 8 bytes
- return address 是 8 bytes
- 通用寄存器如 `rax` 是 8 bytes
- `push` / `pop` 一个 64-bit 寄存器会移动 `rsp` 8 bytes

例如：

```asm
push rax
```

可以理解成：

```asm
sub rsp, 8
mov [rsp], rax
```

所以图里写：

```text
rsp - 8*1
rsp - 8*2
rsp - 8*3
```

意思是按 8 bytes 一格画 stack slot。不是说内存只能 8 bytes 一次访问，也不是说每个变量真实都必须占 8 bytes。

比如一个 8-byte stack slot 可以这样理解：

```text
slot 1: [rsp - 8,  rsp - 1]
slot 2: [rsp - 16, rsp - 9]
slot 3: [rsp - 24, rsp - 17]
```

## 2.1. Little-endian 和 big-endian

Endian 只决定一个多字节值在内存里的 byte 顺序。

如果一个 64-bit 值是：

```text
0x1122334455667788
```

按数字本身的写法看，左边是高有效 byte，右边是低有效 byte：

```text
高有效 byte                      低有效 byte
11 22 33 44 55 66 77 88
```

little-endian 是低地址放低有效 byte：

```text
低地址 -> 高地址
88 77 66 55 44 33 22 11
```

big-endian 是低地址放高有效 byte：

```text
低地址 -> 高地址
11 22 33 44 55 66 77 88
```

x86-64 使用 little-endian。所以如果从低地址开始读内存，看到的 byte 顺序会和数字正常写法反过来。

注意把两个“高低”分开：

- 地址高低：`0x1000` 比 `0x1008` 低。
- 数字位高低：`0x11` 是最高有效 byte，`0x88` 是最低有效 byte。

还有一个常见困扰：图上“上/下”不一定等于地址“高/低”。有些图把高地址画在上面，有些图把高地址画在下面。判断时不要靠屏幕上下，靠标签：

```text
低地址 -> 高地址
```

或者：

```text
高地址
...
低地址
```

只要图没有明确标地址方向，就先不要假设“上面就是高地址”或者“下面就是高地址”。

## 3. `rsp`、return address、caller/callee 边界

`rsp` 是 stack pointer，保存当前栈顶地址。x86-64 的栈通常向低地址增长：`push` 让 `rsp` 变小，`pop` 让 `rsp` 变大。

执行：

```asm
call foo
```

CPU 会把 return address 压到栈上，然后跳到 `foo`。进入 `foo` 时：

```text
低地址
[ callee 可以使用的空间 ]
[ return address ]   <- rsp 指向这里
[ caller 已经在用的空间 ]
高地址
```

return address 是调用约定里的边界和返回凭证。`ret` 会从 `[rsp]` 取出这个地址，并让程序跳回 caller。

但它不是硬件隔离。同一个进程里的函数共享同一片栈。callee 如果数组越界，就可能覆盖 return address，导致 `ret` 跳到错误地址，常见结果是 segmentation fault，也可能成为攻击入口。

## 4. 为什么不能乱写内存

寄存器很少，内存很大，但内存必须被进程里的不同组件共享：

- 函数调用栈
- 局部变量
- heap 对象
- 全局变量
- allocator / runtime / garbage collector
- 代码段

所以程序不能随机写一个地址：

```c
int *p = (int *)0x12345678;
*p = 42;
```

原因有两个：

1. 那块内存可能属于当前进程里的别的组件，乱写会破坏它的不变量。
2. 那个虚拟地址可能没有映射，或者当前进程没有权限访问，硬件和 OS 会触发 fault，程序通常会 segmentation fault。

## 5. 教学版编译器里的 `rax`

一个简单编译器可以只把 `rax` 当临时计算器：

```text
stack slot 保存变量
rax 保存当前正在算的值
```

例如：

```text
let x = 10
in add1(x)
```

可能编译成：

```asm
mov rax, 10
mov [rsp - 8*1], rax
mov rax, [rsp - 8*1]
add rax, 1
```

这里有重复：

```asm
mov [rsp - 8*1], rax
mov rax, [rsp - 8*1]
```

因为 `x` 刚存进 stack slot，下一步马上又读出来。更短可以写成：

```asm
mov rax, 10
add rax, 1
```

甚至如果做 constant propagation，可以直接：

```asm
mov rax, 11
```

课件里的版本故意保守：每个 `let` 先把结果放到 `rax`，再存进变量对应的 stack slot；之后用变量时，再从 stack slot 读回 `rax`。这样规则统一，容易实现。

真实优化编译器会做更多事：

- 删除多余 load/store
- 常量传播
- 复用已经在寄存器里的值
- 做寄存器分配，尽量少访问内存

教学版先求正确，不急着优化。
