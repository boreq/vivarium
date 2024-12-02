.PHONY: ci
ci: test-verbose clippy

.PHONY: cleanup
cleanup: fix clippy fmt

.PHONY: test
test:
	cargo test

.PHONY: test-verbose
test-verbose:
	RUST_BACKTRACE=1 cargo test -- --nocapture

.PHONY: clippy
clippy:
	cargo clippy -- -D warnings

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: fix
fix:
	cargo fix --allow-staged
	cargo clippy --fix --allow-staged
	cargo fmt

.PHONY: push
push:
	#rsync --progress -z -r --exclude target --delete . terrarium:~/vivarium_assistant_repository
	rsync --progress -z ./target/arm-unknown-linux-gnueabihf/release/vivarium_assistant terrarium:~/vivarium_assistant_cross_compile_release
	rsync --progress -z ./target/arm-unknown-linux-gnueabihf/debug/vivarium_assistant terrarium:~/vivarium_assistant_cross_compile_debug

RPI_RUSTFLAGS=-C target-cpu=generic -C linker=./wrapper-arm-linux-gnueabihf.sh
RPI_TARGET=arm-unknown-linux-gnueabihf

.PHONY: cross-compile
cross-compile:
	RUSTFLAGS='$(RPI_RUSTFLAGS)' cargo build --target=$(RPI_TARGET) --bin=vivarium_assistant
	RUSTFLAGS='$(RPI_RUSTFLAGS)' cargo build --target=$(RPI_TARGET) --bin=vivarium_assistant --release
	#RUSTFLAGS='-C target-cpu=generic' rustc --print=cfg --target=arm-unknown-linux-gnueabihf 
	#RUSTFLAGS='-C target-cpu=generic' rustc --target=arm-unknown-linux-gnueabihf -Z unstable-options --print cfg

.PHONY: run
run:
	RUST_LOG=debug cargo run --bin vivarium_assistant --features=not_raspberry_pi example_config.toml
