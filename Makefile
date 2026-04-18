.PHONY: build clean help

build:
	wasm-pack build --target web --out-dir ../../extension/pkg crates/vim_core

clean:
	rm -rf extension/pkg
	cargo clean
