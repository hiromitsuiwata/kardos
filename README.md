# kardos

## 参照

- <https://os.phil-opp.com/ja/freestanding-rust-binary/>

```bash
cargo install bootimage
rustup component add llvm-tools-preview
qemu-system-x86_64 -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-blog_os.bin
```
