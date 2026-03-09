CORE_DIR  = core-parser
WASM_OUT  = packages\mdx-next\omni-core

.PHONY: build test build-web setup clean

build: test build-web

test:
	cd $(CORE_DIR) && cargo run --bin test_ast
	cd $(CORE_DIR) && cargo run --bin test_errors
	cd $(CORE_DIR) && cargo run --bin test_perf

build-web:
	cd $(CORE_DIR) && wasm-pack build --target bundler --release --features wasm
	cmd /c "if not exist $(WASM_OUT) mkdir $(WASM_OUT)"
	cmd /c "xcopy /E /Y /I $(CORE_DIR)\pkg\* $(WASM_OUT)\"

setup:
	npm install
	cd tests/next-sandbox && npm install

clean:
	cd $(CORE_DIR) && cargo clean
	cmd /c "if exist $(CORE_DIR)\pkg rmdir /S /Q $(CORE_DIR)\pkg"