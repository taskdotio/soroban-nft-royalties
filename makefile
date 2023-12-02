default: build

all: test

test: build
	cargo test
	cargo test --features testutils

build:
	soroban contract build

fmt:
	cargo fmt --all

clean:
	cargo clean