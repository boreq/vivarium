MAIN_BINARY_NAME=vivarium_assistant

.PHONY: ci
ci: test-verbose clippy build-release fix fmt check-repository-unchanged

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
	cargo fix --allow-staged --allow-dirty
	cargo clippy --fix --allow-staged --allow-dirty
	cargo fmt

.PHONY: build-release
build-release:
	cargo build --bin $(MAIN_BINARY_NAME) --release

.PHONY: run
run:
	RUST_LOG=debug cargo run --bin $(MAIN_BINARY_NAME) --features=not_raspberry_pi example_config.toml

.PHONY: tools
tools:
	cargo install cross --git https://github.com/cross-rs/cross

.PHONY: check-repository-unchanged
check-repository-unchanged:
	./tools/check_repository_unchanged.sh

CROSS_COMPILATION_RPI_RUSTFLAGS=-C target-cpu=arm1176jz-s
CROSS_COMPILATION_RPI_TARGET=arm-unknown-linux-gnueabihf
CROSS_COMPILATION_RPI_COMMAND=RUSTFLAGS='$(CROSS_COMPILATION_RPI_RUSTFLAGS)' CROSS_CONTAINER_ENGINE=podman cross build --target=$(CROSS_COMPILATION_RPI_TARGET) 

.PHONY: update
update: cross-compile-release cross-compile-release-push restart

.PHONY: restart
restart: 
	ssh terrarium 'sudo systemctl restart vivarium-assistant.service; sleep 5; systemctl status vivarium-assistant.service'

.PHONY: cross-compile
cross-compile: cross-compile-debug cross-compile-release

.PHONY: cross-compile-push
cross-compile-push: cross-compile-debug-push cross-compile-release-push

.PHONY: cross-compile-release
cross-compile-release:
	$(CROSS_COMPILATION_RPI_COMMAND) --bin=$(MAIN_BINARY_NAME) --release

.PHONY: cross-compile-release-push
cross-compile-release-push:
	rsync --progress -z ./target/$(CROSS_COMPILATION_RPI_TARGET)/release/$(MAIN_BINARY_NAME) terrarium:~/vivarium_assistant_cross_compile_release

.PHONY: cross-compile-debug
cross-compile-debug:
	$(CROSS_COMPILATION_RPI_COMMAND) --bin=$(MAIN_BINARY_NAME)

.PHONY: cross-compile-debug-push
cross-compile-debug-push:
	rsync --progress -z ./target/$(CROSS_COMPILATION_RPI_TARGET)/debug/$(MAIN_BINARY_NAME) terrarium:~/vivarium_assistant_cross_compile_debug
