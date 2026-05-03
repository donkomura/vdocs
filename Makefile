.PHONY: build clean test test-rust test-js help

build:
	wasm-pack build --target web --out-dir ../../extension/pkg crates/vim_core

clean:
	rm -rf extension/pkg
	cargo clean

test: test-rust test-js

test-rust:
	cargo test

test-js:
	npm --prefix extension test

help:
	@echo "Available targets:"
	@echo "  build      Build the Wasm module into extension/pkg/"
	@echo "  clean      Remove extension/pkg/ and run cargo clean"
	@echo "  test       Run all tests (Rust + JS)"
	@echo "  test-rust  Run cargo test"
	@echo "  test-js    Run JS unit tests under extension/"
	@echo "  help       Show this message"
