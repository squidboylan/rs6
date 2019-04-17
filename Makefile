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

KERNEL_SOURCE_FILES=$(shell find kernel/src)
KERNEL_LIB=target/i386-os/release/libkernel.a

rsv6.img: bootblock Cargo.toml kernel_bin
	dd if=/dev/zero of=rsv6.img count=10000
	dd if=bootblock of=rsv6.img conv=notrunc
	dd if=kernel_bin of=rsv6.img seek=1 conv=notrunc

kernel_bin: kernel/Cargo.toml kernel/entry.S i386-os.json $(KERNEL_SOURCE_FILES)
	nasm -f elf32 kernel/entry.S -o entry.o
	RUSTFLAGS="-C opt-level=z" cargo xbuild --release --target i386-os.json -p kernel
	$(LD) $(LDFLAGS) -T kernel.ld -o kernel_bin entry.o $(KERNEL_LIB) -b binary
	$(OBJDUMP) -S kernel_bin > kernel_bin.asm
	$(OBJDUMP) -t kernel_bin | sed '1,/SYMBOL TABLE/d; s/ .* / /; /^$$/d' > kernel_bin.sym

BOOT_SOURCE_FILES=$(shell find bootloader/src)
BOOTLOADER_LIB=target/i386-os/release/libbootloader.a

bootblock: bootloader/bootasm.S $(BOOT_SOURCE_FILES)
	RUSTFLAGS="-C opt-level=z" cargo xbuild --release --target i386-os.json -p bootloader
	nasm -f elf32 bootloader/bootasm.S -o bootasm.o
	$(LD) $(LDFLAGS) -N -e start -Ttext 0x7C00 -o bootblock.o bootasm.o $(BOOTLOADER_LIB)
	$(OBJDUMP) -S bootblock.o > bootblock.asm
	$(OBJCOPY) -S -O binary -j .text bootblock.o bootblock
	./sign.sh

.PRECIOUS: %.o

clean:
	rm -f *.tex *.dvi *.idx *.aux *.log *.ind *.ilg \
	*.o *.d *.asm *.sym vectors.S bootblock entryother \
	initcode initcode.out rsv6.img kernel_bin fs.img kernelmemfs \
	rsv6memfs.img mkfs .gdbinit \
	$(UPROGS)
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

qemu: rsv6.img
	$(QEMU) -serial mon:stdio $(QEMUOPTS)

qemu-nox: rsv6.img
	$(QEMU) -nographic $(QEMUOPTS)

.gdbinit: .gdbinit.tmpl
	sed "s/localhost:1234/localhost:$(GDBPORT)/" < $^ > $@

qemu-gdb: rsv6.img .gdbinit
	@echo "*** Now run 'gdb'." 1>&2
	$(QEMU) -serial mon:stdio $(QEMUOPTS) -S $(QEMUGDB)

qemu-nox-gdb: rsv6.img .gdbinit
	@echo "*** Now run 'gdb'." 1>&2
	$(QEMU) -nographic $(QEMUOPTS) -S $(QEMUGDB)
