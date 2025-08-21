#####
## BUILD
#####
# CC=riscv64-unknown-linux-gnu-g++
CC=riscv64-linux-gnu-g++
CFLAGS=-Wall -Wextra -pedantic -Wextra -O0 -g -std=c++17
CFLAGS+=-static -ffreestanding -nostdlib -fno-rtti -fno-exceptions
CFLAGS+=-march=rv64gc -mabi=lp64d
INCLUDES=
LINKER_SCRIPT=-Tsrc/lds/virt.lds -Wl,--build-id=none
TYPE=debug
RUST_TARGET=./target/riscv64gc-unknown-none-elf/$(TYPE)
LIBS=-L$(RUST_TARGET)
SOURCES_ASM=$(wildcard src/asm/*.S)
# wtf is -lsos
LIB=-lshmageOS -lgcc
OUT=os.elf

#####
## QEMU
#####
QEMU=qemu-system-riscv64
MACH=virt
CPU=rv64
CPUS=4
MEM=128M
DRIVE=hdd.dsk

# The problem: i think -lsos is supposed to link to some static libirary file with the symbol name for
# kernel_main, but idk this isn't in there
# do we even need the hdd.dsk?
# -drive if=none,format=raw,file=$(DRIVE),id=foo
all:
	cargo build 
	$(CC) $(CFLAGS) $(LINKER_SCRIPT) $(INCLUDES) -o $(OUT) $(SOURCES_ASM) $(LIBS) $(LIB)
run: all
	$(QEMU) -machine $(MACH) -cpu $(CPU) -smp $(CPUS) -m $(MEM)  -nographic -serial mon:stdio -bios none -kernel $(OUT)
create_boot_image:
	echo "TODO: make it so you can link to a bootloader"
serial_console:
	sudo screen /dev/ttyACM0


.PHONY: clean
clean:
	cargo clean
	rm -f $(OUT)
