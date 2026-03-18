import type { AstNode } from "./MDXServerRenderer";
import { createRequire } from "module";
import { fileURLToPath } from "url";
import { dirname, resolve } from "path";
import { MdxBinaryDecoder } from "./utils/binaryDecoder";

declare const __non_webpack_require__: NodeRequire | undefined;

const _require = createRequire(import.meta.url);
const nativeRequire: NodeRequire =
  typeof __non_webpack_require__ !== "undefined"
    ? __non_webpack_require__
    : _require;

let _nativeModule: any = null;

async function getNativeModule(): Promise<any> {
  if (_nativeModule) return _nativeModule;

  const platformPackages = [
    "@toaq-oss/omni-mdx-darwin-arm64",
    "@toaq-oss/omni-mdx-darwin-x64",
    "@toaq-oss/omni-mdx-linux-x64-gnu",
    "@toaq-oss/omni-mdx-win32-x64-msvc",
  ];

  for (const pkg of platformPackages) {
    try {
      const native = nativeRequire(pkg);
      if (typeof native.parseToJson === "function") {
        _nativeModule = native;
        return _nativeModule;
      } else if (typeof native.parse_to_json === "function") {
        _nativeModule = native;
        return _nativeModule;
      }
    } catch { /* not installed on this platform */ }
  }

  // Fallback local
  try {
    const __dir    = dirname(fileURLToPath(import.meta.url));
    const nativeDir = resolve(__dir, "../native");
    const fs = nativeRequire("fs") as typeof import("fs");

    if (fs.existsSync(nativeDir)) {
      const files = fs.readdirSync(nativeDir).filter((f: string) => f.endsWith(".node"));
      const platform = process.platform ?? "";
      const arch     = process.arch     ?? "";
      const match    = files.find((f: string) => f.includes(platform) && f.includes(arch))
                    ?? files[0];

      if (match) {
        const native = nativeRequire(resolve(nativeDir, match));
        _nativeModule = native;
        if (typeof native.parseToJson === "function") {
          return _nativeModule;
        } else if (typeof native.parse_to_json === "function") {
          return _nativeModule;
        }
      }
    }
  } catch { /* native/ missing */ }

  throw new Error(
    `[toaq-oss/omni-mdx] Native parser not found for this platform.\n` +
    `  Tried npm packages: ${platformPackages.join(", ")}\n` +
    `  ➜ Install the prebuilt package (once published):\n` +
    `      npm install @toaq-oss/omni-mdx-darwin-arm64   # Mac M1/M2\n` +
    `      npm install @toaq-oss/omni-mdx-linux-x64-gnu  # Linux/Vercel\n` +
    `      npm install @toaq-oss/omni-mdx-win32-x64-msvc # Windows\n`
  );
}

function getParseFn(native: any): (mdx: string) => any {
  if (typeof native.parseToJson === "function") return native.parseToJson;
  if (typeof native.parse_to_json === "function") return native.parse_to_json;
  if (typeof native.parse === "function") return native.parse;
  throw new Error("[toaq-oss/omni-mdx] Native parser lacks a valid parse function.");
}

export async function parseMdx(mdx: string): Promise<AstNode[]> {
  const native = await getNativeModule();
  const parse = getParseFn(native);
  let result: any;
  
  try {
    result = parse(mdx);
  } catch (err: any) {
    throw new MDXParseError(err?.message ?? String(err), mdx);
  }

  if (result instanceof Uint8Array || Buffer.isBuffer(result)) {
    const decoder = new MdxBinaryDecoder(result);
    return decoder.decode();
  }

  if (typeof result === "string") {
    try { return JSON.parse(result) as AstNode[]; } catch { throw new Error("Invalid JSON string."); }
  }

  if (typeof result === "object" && result !== null) {
    if (typeof result.toJson === "function") {
      try {
        const jsonString = result.toJson();
        return JSON.parse(jsonString) as AstNode[];
      } catch (e) {
        throw new Error("[toaq-oss/omni-mdx] Failed to serialize MdxAst to JSON.");
      }
    }
    
    if (typeof result.to_json === "function") {
      try { return JSON.parse(result.to_json()) as AstNode[]; } catch (e) {}
    }
    
    if (Array.isArray(result)) return result as AstNode[];
  }

  throw new Error("[toaq-oss/omni-mdx] Unrecognized return format from Rust parser. Available properties: " + Object.keys(result.__proto__ || result).join(", "));
}

export function parseMdxSync(mdx: string): AstNode[] {
  const native = getNativeModule();
  const parse = getParseFn(native);

  let result: any;
  try {
    result = parse(mdx);
  } catch (err: any) {
    throw new MDXParseError(err?.message ?? String(err), mdx);
  }

  if (result instanceof Uint8Array || Buffer.isBuffer(result)) {
    const decoder = new MdxBinaryDecoder(result);
    return decoder.decode();
  }
  
  if (typeof result === "string") {
    try { return JSON.parse(result) as AstNode[]; } catch { throw new Error("Invalid JSON"); }
  }

  if (typeof result === "object" && result !== null && typeof result.toJson === "function") {
    try { return JSON.parse(result.toJson()) as AstNode[]; } catch { throw new Error("Invalid JSON"); }
  }

  if (typeof result === "object" && result !== null && typeof result.to_json === "function") {
    try { return JSON.parse(result.to_json()) as AstNode[]; } catch { throw new Error("Invalid JSON"); }
  }

  throw new Error("[toaq-oss/omni-mdx] Unrecognized return format from Rust parser.");
}

export async function compileToJsx(mdx: string): Promise<string> {
  const native = await getNativeModule();
  
  const compileFn = native.compileToJsx || native.compile_to_jsx;
  
  if (typeof compileFn !== "function") {
    throw new Error("[toaq-oss/omni-mdx] compileToJsx is not supported by this native module version.");
  }
  
  try {
    return compileFn(mdx);
  } catch (err: any) {
    throw new MDXParseError(err?.message ?? String(err), mdx);
  }
}

export function compileToJsxSync(mdx: string): string {
  if (!_nativeModule) throw new Error("[toaq-oss/omni-mdx] compileToJsxSync() called before init.");
  
  const compileFn = _nativeModule.compileToJsx || _nativeModule.compile_to_jsx;
  if (typeof compileFn !== "function") {
    throw new Error("[toaq-oss/omni-mdx] compileToJsx is not supported by this native module version.");
  }
  
  try {
    return compileFn(mdx);
  } catch (err: any) {
    throw new MDXParseError(err?.message ?? String(err), mdx);
  }
}

export class MDXParseError extends Error {
  readonly source: string;
  constructor(message: string, source: string) {
    super(`MDX parse error: ${message}`);
    this.name   = "MDXParseError";
    this.source = source;
  }
}