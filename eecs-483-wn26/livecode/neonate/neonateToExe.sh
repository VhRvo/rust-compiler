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


# Our first compiler pipeline

# Number literal
# --[ compiler ]--> x86_64
# --[ nasm ]--> object file
# --[ ar ]--> archive file
# --[ ld ]--> Executable     links with our "runtime system" in stub.rs

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
