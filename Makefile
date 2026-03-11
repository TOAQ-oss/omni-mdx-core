CORE_DIR = core-parser
WASM_OUT = packages/mdx-next/omni-core

.PHONY: build test build-web setup clean

build: test build-web

test:
	cd $(CORE_DIR) && cargo run --bin test_ast
	cd $(CORE_DIR) && cargo run --bin test_errors
	cd $(CORE_DIR) && cargo run --bin test_perf

build-web:
	cd $(CORE_DIR) && wasm-pack build --target web --release --features wasm
	mkdir -p $(WASM_OUT)
	cp -r $(CORE_DIR)/pkg/* $(WASM_OUT)/
	rm -f $(WASM_OUT)/.gitignore

build-python:
	cd $(CORE_DIR) && maturin develop --release --features python
	pip install -e $(PYTHON_PKG)
	
setup:
	npm install
	cd tests/next-sandbox && npm install

clean:
	cd $(CORE_DIR) && cargo clean
	rm -rf $(CORE_DIR)/pkg