clean:
	cargo clean
	rm -rf img
	rm memory_map.csv

loader:
	cd kardos-loader && cargo build

kernel:
	cd kardos-kernel && cargo build

image: loader kernel
	rm -rf ./img
	mkdir ./img
	qemu-img create -f raw ./img/disk.img 200M
	mkfs.fat -n 'KARDOS' -s 2 -f 2 -R 32 -F 32 ./img/disk.img
	rm -rf ./mnt
	mkdir -p mnt
	sudo mount -o loop ./img/disk.img mnt
	sudo mkdir -p mnt/EFI/BOOT
	sudo cp ./target/x86_64-unknown-uefi/debug/kardos-loader.efi ./mnt/EFI/BOOT/BOOTX64.EFI
	sudo cp ./target/x86_64-unknown-none-elf/debug/kardos-kernel.elf ./mnt/kernel.elf
	sudo umount mnt

run_image: image
	cp ./OVMF/OVMF_VARS.fd ./OVMF/OVMF_VARS_MOD.fd
	qemu-system-x86_64 \
    	-drive if=pflash,format=raw,readonly,file=./OVMF/OVMF_CODE.fd \
    	-drive if=pflash,format=raw,file=./OVMF/OVMF_VARS_MOD.fd \
    	-hda ./img/disk.img
	# copy memory map
	sudo mount -o loop ./img/disk.img mnt
	cp mnt/memory_map.csv ./
	sudo umount mnt

run_efi: loader kernel
	cp ./OVMF/OVMF_VARS.fd ./OVMF/OVMF_VARS_MOD.fd
	rm -rf ./mnt
	mkdir -p ./mnt/EFI/BOOT
	cp ./target/x86_64-unknown-uefi/debug/kardos-loader.efi ./mnt/EFI/BOOT/BOOTX64.EFI
	cp ./target/x86_64-unknown-none-elf/debug/kardos-kernel.elf ./mnt/kernel.elf
	qemu-system-x86_64 \
	    -drive if=pflash,format=raw,readonly,file=./OVMF/OVMF_CODE.fd \
		-drive if=pflash,format=raw,file=./OVMF/OVMF_VARS_MOD.fd \
		-hda fat:rw:./mnt

