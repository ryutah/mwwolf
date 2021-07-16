FROM rust:1.52-buster

RUN apt update && apt install -y nodejs && \
	cargo install wasm-bindgen-cli && \
	rustup component add clippy rustfmt && \
	rustup target add wasm32-unknown-unknown && \
	rm -rf /usr/local/cargo/registry
