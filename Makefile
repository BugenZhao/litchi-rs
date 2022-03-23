default: qemu

efi/QEMU_EFI.fd:
	@mkdir -p efi
	curl -L https://github.com/rust-osdev/ovmf-prebuilt/releases/download/v0.20211216.143%2Bg267a92fef3/OVMF-pure-efi.fd -o efi/QEMU_EFI.fd

build:
	cd litchi-kernel && cargo build
	cd litchi-boot && cargo build

efi/EFI/BOOT/BOOTX64.efi: build
	@mkdir -p efi/EFI/BOOT
	cp target/x86_64-unknown-uefi/debug/litchi-boot.efi efi/EFI/BOOT/BOOTX64.efi

qemu: efi/QEMU_EFI.fd efi/EFI/BOOT/BOOTX64.efi
	rm -f efi/NvVars
	qemu-system-x86_64 \
		-m 256M \
		-gdb tcp::1234 \
		-nographic \
		-bios efi/QEMU_EFI.fd \
		-drive format=raw,file=fat:rw:./efi/ \

clean:
	cargo clean
	rm -rf efi
