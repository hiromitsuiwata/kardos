#!/usr/bin/bash

# create efi
#cargo +nightly build --target x86_64-unknown-uefi
cargo build
if [ $? -ne 0 ]; then
    echo "Build failed. exiting."
    exit 1
fi

# qemu起動
# 書き込まれる領域をコピー
cp ./OVMF/OVMF_VARS.fd ./OVMF/OVMF_VARS_MOD.fd
# efiファイルをコピー
rm -rf ./mnt
mkdir -p ./mnt/EFI/BOOT
cp ./target/x86_64-unknown-uefi/debug/kardos.efi ./mnt/EFI/BOOT/BOOTX64.EFI
# 起動
qemu-system-x86_64 -drive if=pflash,format=raw,readonly,file=./OVMF/OVMF_CODE.fd -drive if=pflash,format=raw,file=./OVMF/OVMF_VARS_MOD.fd -hda fat:rw:./mnt
