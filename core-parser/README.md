# mdx-parser

A fast MDX parser written in Rust, distributable as:

| Target     | Format           | Tool                  |
|------------|------------------|-----------------------|
| **NPM**    | `.wasm` + JS glue | `wasm-pack`          |
| **Python** | `.so` shared lib | `cffi` / `ctypes`    |
| **Dart**   | `.so` shared lib | `dart:ffi`           |
| **Native** | `rlib`           | `cargo`              |

---

## Building

### Prerequisites

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

---

### NPM / JavaScript (WASM)

```bash
wasm-pack build --target web --release
# → ./pkg/  contains mdx_parser.js + mdx_parser_bg.wasm + TypeScript types
```

```ts
import init, { parse_mdx_to_json } from "./pkg/mdx_parser.js";

await init();
const ast = JSON.parse(parse_mdx_to_json("# Hello\n<Alert type='info' />"));
console.log(ast);
```

Errors surface as native JavaScript `Error` objects — no need to check a separate error field.

---

### Python (cffi)

```bash
cargo build --release --features ffi
# → target/release/libmdx_parser.so  (Linux)
# → target/release/libmdx_parser.dylib  (macOS)
```

```python
import cffi, json

ffi = cffi.FFI()
ffi.cdef("""
    char* mdx_parse(const char* input);
    void  mdx_free (char* ptr);
""")

lib = ffi.dlopen("./target/release/libmdx_parser.so")

def parse_mdx(src: str) -> dict:
    ptr = lib.mdx_parse(src.encode())
    try:
        raw = ffi.string(ptr).decode()
        result = json.loads(raw)
        if "error" in result:
            raise ValueError(result["error"])
        return result
    finally:
        lib.mdx_free(ptr)

ast = parse_mdx("# Hello\n<Alert />")
print(ast)
```

---

### Dart (dart:ffi)

```bash
cargo build --release --features ffi
# → target/release/libmdx_parser.so
```

```dart
import 'dart:ffi';
import 'dart:convert';
import 'package:ffi/ffi.dart';

final lib = DynamicLibrary.open('libmdx_parser.so');

final _mdxParse = lib.lookupFunction<
  Pointer<Utf8> Function(Pointer<Utf8>),
  Pointer<Utf8> Function(Pointer<Utf8>)
>('mdx_parse');

final _mdxFree = lib.lookupFunction<
  Void Function(Pointer<Utf8>),
  void Function(Pointer<Utf8>)
>('mdx_free');

Map<String, dynamic> parseMdx(String input) {
  final inputPtr = input.toNativeUtf8();
  final resultPtr = _mdxParse(inputPtr);
  calloc.free(inputPtr);
  try {
    final json = resultPtr.toDartString();
    final decoded = jsonDecode(json) as Map<String, dynamic>;
    if (decoded.containsKey('error')) throw Exception(decoded['error']);
    return decoded;
  } finally {
    _mdxFree(resultPtr);
  }
}
```

---

## AST format

```jsonc
[
  {
    "node_type": "h1",
    "children": [
      { "node_type": "text", "content": "Hello" }
    ]
  },
  {
    "node_type": "Alert",
    "self_closing": true,
    "attributes": {
      "type": { "kind": "text",       "value": "info" },
      "count": { "kind": "expression","value": "42"   },
      "disabled": { "kind": "boolean" }
    }
  }
]
```

### `AttrValue` kinds

| `kind`       | Example source           | Meaning                      |
|--------------|--------------------------|------------------------------|
| `text`       | `prop="hello"`           | Quoted string literal        |
| `expression` | `prop={someVar + 1}`     | Raw JS expression (as string)|
| `boolean`    | `disabled`               | Bare attribute (implicit true)|
| `ast`        | *(render-prop subtrees)* | Nested AST                   |

---

## Error handling

All public functions return `Result`/`JsError`/JSON error objects — no panics
in release builds. The FFI layer returns `{"error": "<message>"}` on failure
and always requires the caller to free the pointer with `mdx_free`.