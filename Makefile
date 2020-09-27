# This file is derived from the xv6 Makefile see xv6-LICENSE for a copy of
# the license and copyright info

# Try to infer the correct TOOLPREFIX if not set
ifndef TOOLPREFIX
TOOLPREFIX := $(shell if i386-jos-elf-objdump -i 2>&1 | grep '^elf32-i386$$' >/dev/null 2>&1; \
	then echo 'i386-jos-elf-'; \
	elif objdump -i 2>&1 | grep 'elf32-i386' >/dev/null 2>&1; \
	then echo ''; \
	else echo "***" 1>&2; \
	echo "*** Error: Couldn't find an i386-*-elf version of GCC/binutils." 1>&2; \
	echo "*** Is the directory with i386-jos-elf-gcc in your PATH?" 1>&2; \
	echo "*** If your i386-*-elf toolchain is installed with a command" 1>&2; \
	echo "*** prefix other than 'i386-jos-elf-', set your TOOLPREFIX" 1>&2; \
	echo "*** environment variable to that prefix and run 'make' again." 1>&2; \
	echo "*** To turn off this error, run 'gmake TOOLPREFIX= ...'." 1>&2; \
	echo "***" 1>&2; exit 1; fi)
endif

# If the makefile can't find QEMU, specify its path here
# QEMU = qemu-system-i386

# Try to infer the correct QEMU
ifndef QEMU
QEMU = $(shell if which qemu > /dev/null; \
	then echo qemu; exit; \
	elif which qemu-system-i386 > /dev/null; \
	then echo qemu-system-i386; exit; \
	elif which qemu-system-x86_64 > /dev/null; \
	then echo qemu-system-x86_64; exit; \
	else \
	qemu=/Applications/Q.app/Contents/MacOS/i386-softmmu.app/Contents/MacOS/i386-softmmu; \
	if test -x $$qemu; then echo $$qemu; exit; fi; fi; \
	echo "***" 1>&2; \
	echo "*** Error: Couldn't find a working QEMU executable." 1>&2; \
	echo "*** Is the directory containing the qemu binary in your PATH" 1>&2; \
	echo "*** or have you tried setting the QEMU variable in Makefile?" 1>&2; \
	echo "***" 1>&2; exit 1)
endif

CC = $(TOOLPREFIX)gcc
AS = $(TOOLPREFIX)gas
LD = $(TOOLPREFIX)ld
OBJCOPY = $(TOOLPREFIX)objcopy
OBJDUMP = $(TOOLPREFIX)objdump
# FreeBSD ld wants ``elf_i386_fbsd''
LDFLAGS += -m $(shell $(LD) -V | grep elf_i386 2>/dev/null | head -n 1)

RUST_TARGET_PATH := target/i386-os
RUST_REL_TARGET := $(RUST_TARGET_PATH)/release
RUST_DBG_TARGET := $(RUST_TARGET_PATH)/debug

# To build the Rust code with the 'release' profile, invoke
# make with RELEASE=1:
#
#	$ RELEASE=1 make
#
# Otherwise the debug profile is the default

RELEASE ?= 0

ifeq ($(RELEASE), 1)
	RUST_TARGET := $(RUST_REL_TARGET)
	RELEASE_FLAG := --release
else
	RUST_TARGET := $(RUST_DBG_TARGET)
endif

RUST_BUILD_PREFIX := RUSTFLAGS="-C opt-level=z"
RUST_BOOTLOADER_LIB := $(RUST_TARGET)/libbootloader.a
RUST_KERNEL_LIB := $(RUST_TARGET)/libkernel.a
BOOTBLOCK := bootblock
KERN_ELF := kernel.elf
RSV6_IMG := rsv6.img
LDSCRIPT := kernel.ld

.PHONY: default build

default: build

build: $(RSV6_IMG)

$(RSV6_IMG): $(BOOTBLOCK) $(KERN_ELF)
	dd if=/dev/zero of=$@ count=10000
	dd if=$(BOOTBLOCK) of=$@ conv=notrunc
	dd if=$(KERN_ELF) of=$@ seek=1 conv=notrunc

$(BOOTBLOCK): bootblock.o
	$(OBJDUMP) -S $^ > bootblock.asm
	$(OBJCOPY) -S -O binary -j .text bootblock.o $@
	./sign.sh

bootblock.o: bootasm.o $(RUST_BOOTLOADER_LIB)
	$(LD) $(LDFLAGS) -N -e start -Ttext 0x7C00 -o $@ $^

$(KERN_ELF): $(LDSCRIPT) entry.o $(RUST_KERNEL_LIB)
	$(LD) $(LDFLAGS) -T $(LDSCRIPT) -o $(KERN_ELF) entry.o $(RUST_KERNEL_LIB) -b binary
	$(OBJDUMP) -S $@ > $@.asm
	$(OBJDUMP) -t $@ | sed '1,/SYMBOL TABLE/d; s/ .* / /; /^$$/d' > $@.sym

entry.o: kernel/entry.S
	nasm -f elf32 $^ -o $@

bootasm.o: bootloader/bootasm.S
	nasm -f elf32 $^ -o $@

BOOTLOADER_SOURCE := $(shell find bootloader)

$(RUST_BOOTLOADER_LIB): $(BOOTLOADER_SOURCE)
	$(RUST_BUILD_PREFIX) cargo xbuild $(RELEASE_FLAG) -p bootloader

KERNEL_SOURCE := $(shell find kernel)

$(RUST_KERNEL_LIB): $(KERNEL_SOURCE)
	$(RUST_BUILD_PREFIX) cargo xbuild $(RELEASE_FLAG) -p kernel

.PRECIOUS: %.o

clean:
	rm -f *.o *.d *.asm *.sym bootblock entryother $(RSV6_IMG) $(KERN_ELF) .gdbinit
	cargo clean

# try to generate a unique GDB port
GDBPORT = $(shell expr `id -u` % 5000 + 25000)
# QEMU's gdb stub command line changed in 0.11
QEMUGDB = $(shell if $(QEMU) -help | grep -q '^-gdb'; \
	then echo "-gdb tcp::$(GDBPORT)"; \
	else echo "-s -p $(GDBPORT)"; fi)
ifndef CPUS
CPUS := 2
endif
QEMUOPTS = -drive file=rsv6.img,index=0,media=disk,format=raw -smp $(CPUS) -m 512 $(QEMUEXTRA)

qemu: $(RSV6_IMG)
	$(QEMU) -serial mon:stdio $(QEMUOPTS)

qemu-curses: $(RSV6_IMG)
	$(QEMU) -display curses $(QEMUOPTS)

qemu-nox: $(RSV6_IMG)
	$(QEMU) -nographic $(QEMUOPTS)

.gdbinit: .gdbinit.tmpl
	sed "s/localhost:1234/localhost:$(GDBPORT)/" < $^ > $@

qemu-gdb: $(RSV6_IMG) .gdbinit
	@echo "*** Now run 'gdb'." 1>&2
	$(QEMU) -serial mon:stdio $(QEMUOPTS) -S $(QEMUGDB)

qemu-nox-gdb: $(RSV6_IMG) .gdbinit
	@echo "*** Now run 'gdb'." 1>&2
	$(QEMU) -nographic $(QEMUOPTS) -S $(QEMUGDB)
