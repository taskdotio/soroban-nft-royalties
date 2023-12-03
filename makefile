default: build

all: test

test: build
	cargo test

build:
	soroban contract build

fmt:
	cargo fmt --all

clean:
	cargo clean