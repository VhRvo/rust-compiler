; macOS x86_64. On Apple Silicon this runs through Rosetta.
; nasm -fmacho64 hello.asm -o hello.o
; /usr/bin/ld -static -arch x86_64 -platform_version macos 11.0 $(xcrun --show-sdk-version) -e start hello.o -o hello
; ./hello

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

nothing:
