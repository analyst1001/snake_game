arch ?= x86_64
game := build/game-$(arch).bin
iso := build/game-$(arch).iso
target := $(arch)
game_lib := target/$(target)/debug/libsnake_game.a

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.asm)
assembly_object_files := $(patsubst src/arch/$(arch)/%.asm, \
	build/arch/$(arch)/%.o, $(assembly_source_files))

.PHONY: all clean run iso gamebin

all: $(game)

clean:
	@rm -r build
	@cargo clean

run: $(iso)
	@qemu-system-$(arch) -cdrom $(iso)

iso: $(iso)

$(iso): $(game) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(game) build/isofiles/boot/game.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(game): gamebin $(game_lib) $(assembly_object_files) $(linker_script)
	@ld -n --gc-sections -T $(linker_script) -o $(game) $(assembly_object_files) $(game_lib)

gamebin:
	@RUST_TARGET_PATH=$(shell pwd)/src/arch/$(arch) xargo build --target=$(arch)

build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -f elf64 $< -o $@
