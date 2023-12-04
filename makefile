default: build

all: test

test: build
	cargo test

build:
	soroban contract build

build-optimized:
	soroban contract build
	soroban contract optimize --wasm ./target/wasm32-unknown-unknown/release/collectible.wasm --wasm-out ./target/wasm32-unknown-unknown/release/collectible.wasm

fmt:
	cargo fmt --all

clean:
	cargo clean