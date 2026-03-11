CORE_DIR = core-parser
WASM_PKG_DIR = packages/mdx-next/parser-core
PYTHON_PKG = packages/mdx-python

.PHONY: build test build-wasm build-python setup clean

build: setup test build-wasm build-python

test:
	cd $(CORE_DIR) && cargo test --release

build-wasm:
	cd $(CORE_DIR) && cargo build --release --target wasm32-unknown-unknown
	mkdir -p $(WASM_PKG_DIR)
	wasm-bindgen target/wasm32-unknown-unknown/release/mdx_parser.wasm \
		--out-dir $(WASM_PKG_DIR) \
		--target bundler \
		--typescript
	rm -f $(WASM_PKG_DIR)/.gitignore

build-python:
	cd $(CORE_DIR) && maturin develop --release --features python

setup:
	npm install

clean:
	cd $(CORE_DIR) && cargo clean
	rm -rf $(WASM_PKG_DIR)