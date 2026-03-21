CORE_DIR     = core-parser
MDX_NEXT_DIR = packages/mdx-next
PYTHON_PKG   = packages/mdx-python
NATIVE_DIR   = $(MDX_NEXT_DIR)/native
WASM_DIR     = $(MDX_NEXT_DIR)/wasm

.PHONY: build test build-node build-wasm build-python build-mdx-next setup clean

build: setup test build-node build-wasm build-mdx-next build-python

test:
	cd $(CORE_DIR) && cargo test --release

build-node:
	cd $(CORE_DIR) && napi build --platform --release --features node --no-js
	mkdir -p $(NATIVE_DIR)
	cp $(CORE_DIR)/toaq-parser-core.*.node $(NATIVE_DIR)/

build-wasm:
	cd $(CORE_DIR) && wasm-pack build --target web --features wasm
	mkdir -p $(WASM_DIR)
	cp -r $(CORE_DIR)/pkg/* $(WASM_DIR)/

build-mdx-next: build-node build-wasm
	cd $(MDX_NEXT_DIR) && npm run build

build-python:
	cd $(CORE_DIR) && maturin develop --release --features python

setup:
	npm install

clean:
	cd $(CORE_DIR) && cargo clean
	rm -rf $(NATIVE_DIR)
	rm -rf $(WASM_DIR)
	rm -rf $(MDX_NEXT_DIR)/dist
	rm -rf $(MDX_NEXT_DIR)/node_modules