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
