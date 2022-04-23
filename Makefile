PROFILE?=dev
ifeq ($(PROFILE),dev)
	TARGET=debug
else
	TARGET=$(PROFILE)
endif

.PHONY: default build build-users build-kernel build-boot qemu kill clean

default: qemu

efi/QEMU_EFI.fd:
	@mkdir -p efi
	curl -L https://github.com/rust-osdev/ovmf-prebuilt/releases/download/v0.20211216.148%2Bg22130dcd98/OVMF-pure-efi.fd -o efi/QEMU_EFI.fd

build: build-users build-kernel build-boot

build-users:
	cd litchi-user && cargo build --bins --profile $(PROFILE)

build-kernel:
	cd litchi-kernel && cargo build  --profile $(PROFILE)

build-boot:
	cd litchi-boot && cargo build  --profile $(PROFILE)

efi/EFI/BOOT/BOOTX64.efi: build-boot
	@mkdir -p efi/EFI/BOOT
	cp target/x86_64-unknown-uefi/$(TARGET)/litchi-boot.efi efi/EFI/BOOT/BOOTX64.efi

efi/litchi-kernel: build-users build-kernel
	cp target/x86_64-unknown-litchi/$(TARGET)/litchi-kernel efi/litchi-kernel

qemu: efi/QEMU_EFI.fd efi/litchi-kernel efi/EFI/BOOT/BOOTX64.efi
	rm -f efi/NvVars
	qemu-system-x86_64 \
		-smp 4 \
		-m 256M \
		-gdb tcp::1234 \
		-nographic \
		-bios efi/QEMU_EFI.fd \
		-drive format=raw,file=fat:rw:./efi/ \
		-device isa-debug-exit,iobase=0xf4,iosize=0x04 \
		-serial stdio \
		-monitor none \
	; ([ $$? -eq 33 ] && echo "Success") || exit 1

kill:
	killall qemu-system-x86_64

clean:
	cargo clean
	rm -rf efi
