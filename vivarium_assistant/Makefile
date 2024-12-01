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
	rsync --progress -z ./target/arm-unknown-linux-gnueabihf/release/vivarium_assistant terrarium:~/vivarium_assistant

.PHONY: cross-compile
cross-compile:
	cargo build --target=arm-unknown-linux-gnueabihf --bin=vivarium_assistant --release
