.DEFAULT_GOAL := help

FILE ?= source/2021.int
BUILD_DIR := build
CARGO_TARGET_DIR := $(BUILD_DIR)/cargo-target
RUST_TARGET := x86_64-apple-darwin
NASM_FORMAT := macho64

NAME := $(basename $(notdir $(FILE)))
OUT_DIR := $(BUILD_DIR)/$(NAME)
ASM := $(OUT_DIR)/$(NAME).s
OBJ := $(OUT_DIR)/$(NAME).o
LIB := $(OUT_DIR)/libcompiled_code.a
RUNNER := $(OUT_DIR)/$(NAME).run
ABS_FILE := $(abspath $(FILE))

.PHONY: help asm build run clean check-file

help:
	@printf '%s\n' \
		'Usage:' \
		'  make asm FILE=source/2021.int' \
		'  make build FILE=source/2021.int' \
		'  make run FILE=source/2021.int' \
		'  make clean'

asm: check-file $(ASM)
	@printf 'wrote %s\n' '$(ASM)'

build: check-file $(RUNNER)
	@printf 'built %s\n' '$(RUNNER)'

run: check-file $(RUNNER)
	@'$(RUNNER)'

check-file:
	@test -f '$(FILE)' || { echo 'input file not found: $(FILE)'; exit 1; }

$(ASM): $(ABS_FILE) src/main.rs Cargo.toml
	@mkdir -p '$(OUT_DIR)'
	CARGO_TARGET_DIR='$(CARGO_TARGET_DIR)' cargo run --quiet -- '$(ABS_FILE)' > '$@'

$(OBJ): $(ASM)
	nasm -f $(NASM_FORMAT) -o '$@' '$<'

$(LIB): $(OBJ)
	ar rcs '$@' '$<'

$(RUNNER): $(LIB) stub.rs
	rustc --target $(RUST_TARGET) stub.rs -L '$(OUT_DIR)' -o '$@'

clean:
	rm -rf '$(BUILD_DIR)'
