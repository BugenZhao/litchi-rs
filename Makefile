default: qemu

efi/QEMU_EFI.fd:
	@mkdir -p efi
	curl -L https://github.com/rust-osdev/ovmf-prebuilt/releases/download/v0.20211216.143%2Bg267a92fef3/OVMF-pure-efi.fd -o efi/QEMU_EFI.fd

build:
	cargo build -p litchi-kernel --target x86_64-unknown-none
	cargo build -p litchi-boot --target x86_64-unknown-uefi

efi/EFI/BOOT/BOOTX64.efi: build
	@mkdir -p efi/EFI/BOOT
	cp target/x86_64-unknown-uefi/debug/litchi-boot.efi efi/EFI/BOOT/BOOTX64.efi

qemu: efi/QEMU_EFI.fd efi/EFI/BOOT/BOOTX64.efi
	rm -f efi/NvVars
	qemu-system-x86_64 \
		-m 256M \
		-nographic \
		-bios efi/QEMU_EFI.fd \
		-drive format=raw,file=fat:rw:./efi/ \

clean:
	cargo clean
	rm -rf efi
