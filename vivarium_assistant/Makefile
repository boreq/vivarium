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
push: cross-compile-debug-push cross-compile-release-push
	#rsync --progress -z -r --exclude target --delete . terrarium:~/vivarium_assistant_repository

.PHONY: cross-compile
cross-compile: cross-compile-debug cross-compile-release

RPI_RUSTFLAGS=-C target-cpu=generic -C linker=./wrapper-arm-linux-gnueabihf.sh
RPI_TARGET=arm-unknown-linux-gnueabihf

.PHONY: cross-compile-release
cross-compile-release:
	RUSTFLAGS='$(RPI_RUSTFLAGS)' cargo build --target=$(RPI_TARGET) --bin=vivarium_assistant --release

.PHONY: cross-compile-release-push
cross-compile-release-push:
	rsync --progress -z ./target/$(RPI_TARGET)/release/vivarium_assistant terrarium:~/vivarium_assistant_cross_compile_release

.PHONY: cross-compile-debug
cross-compile-debug:
	RUSTFLAGS='$(RPI_RUSTFLAGS)' cargo build --target=$(RPI_TARGET) --bin=vivarium_assistant

.PHONY: cross-compile-debug-push
cross-compile-debug-push:
	rsync --progress -z ./target/$(RPI_TARGET)/debug/vivarium_assistant terrarium:~/vivarium_assistant_cross_compile_debug

.PHONY: run
run:
	RUST_LOG=debug cargo run --bin vivarium_assistant --features=not_raspberry_pi example_config.toml
