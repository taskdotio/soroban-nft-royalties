default: build

all: test

test: build
	cargo test

build:
	soroban contract build

build-optimized:
	soroban contract build
	soroban contract optimize --wasm ./target/wasm32-unknown-unknown/release/collectible.wasm --wasm-out ./target/wasm32-unknown-unknown/release/collectible.wasm
	soroban contract optimize --wasm ./target/wasm32-unknown-unknown/release/deployer.wasm --wasm-out ./target/wasm32-unknown-unknown/release/deployer.wasm

fmt:
	cargo fmt --all

clean:
	cargo clean

launch_standalone:
	docker run -d -it \
      -p 8000:8000 \
      --name stellar-soroban-network \
      stellar/quickstart:testing@sha256:0c756150e7b3c53603fe36bb932c4e7d7ceaef691906b2d3d952771ccc195559 \
      --standalone \
      --enable-soroban-rpc