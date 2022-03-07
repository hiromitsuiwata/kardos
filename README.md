# kardos

## 参照

- <https://zenn.dev/yubrot/scraps/9735639c0c982d>
- <https://os.phil-opp.com/ja/freestanding-rust-binary/>
- <https://github.com/uchan-nos/mikanos-build>

```bash
cargo install bootimage
rustup component add llvm-tools-preview
rustup toolchain install nightly
rustup component add --toolchain nightly rust-src
qemu-system-x86_64 -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-blog_os.bin
```
