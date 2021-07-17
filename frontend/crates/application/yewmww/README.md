# yewmw

## Prerequired

```console
cargo install --locked trunk
cargo install wasm-bindgen-cli
wget https://github.com/WebAssembly/binaryen/releases/download/version_101/binaryen-version_101-x86_64-linux.tar.gz
# untar and place to bin path folder
rustup target add wasm32-unknown-unknown
```

## Start

```console
trunk serve
```
