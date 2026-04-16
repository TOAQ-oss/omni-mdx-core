import type { AstNode } from "./types/MdxAST";
import { createRequire } from "module";
import { fileURLToPath } from "url";
import { dirname, resolve } from "path";
import { MdxBinaryDecoder } from "./utils/binaryDecoder";
import { MdxInput, OmniMdxOptions } from "./types/MdxInput";
import { runUnifiedPipeline, runUnifiedPipelineSync } from "./utils/unifiedBridge";

declare const __non_webpack_require__: NodeRequire | undefined;

const _require = createRequire(import.meta.url);
const nativeRequire: NodeRequire =
  typeof __non_webpack_require__ !== "undefined"
    ? __non_webpack_require__
    : _require;

let _nativeModule: any = null;

function getNativeModuleSync():any {
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
      if (
        typeof native.parseToBinary === "function" ||
        typeof native.parse_to_binary === "function" ||
        typeof native.parseToJson === "function" ||
        typeof native.parse_to_json === "function"
      ) {
        _nativeModule = native;
        return _nativeModule;
      }
    } catch { /* not installed on this platform */ }
  }

  // Local Fallback
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

function normalizeToBuffer(input: any): Buffer {
  if (Buffer.isBuffer(input)) return input;
  if (input instanceof Uint8Array) return Buffer.from(input.buffer, input.byteOffset, input.byteLength);
  if (input && typeof input === 'object' && input.type === 'Buffer' && Array.isArray(input.data)) {
    return Buffer.from(input.data);
  }
  return Buffer.from(String(input), 'utf-8');
}

/**
 * Asynchronously parses raw MDX content into an Abstract Syntax Tree (AST).
 * This is the primary function intended for use in Next.js Server Components and API routes.
 *
 * @param mdx - The raw MDX content as a string, or a binary buffer for optimal performance.
 * @param options - Optional configuration to inject `unified` plugins (e.g., `rehypePlugins`).
 * @returns A promise that resolves to an array of `AstNode` representing the parsed MDX.
 * @throws {MDXParseError} If the MDX syntax is invalid or the underlying native parser fails.
 * @example
 * ```typescript
 * import { parseMdx } from '@toaq-oss/omni-mdx/server';
 * import rehypeHighlight from 'rehype-highlight';
 * 
 * const ast = await parseMdx("# Hello World", {
 *  rehypePlugins: [rehypeHighlight]
 * });
 * ```
 */
export async function parseMdx(mdx: MdxInput, options?: OmniMdxOptions): Promise<AstNode[]> {
  const native = getNativeModuleSync();
  let result: any;
  
  try {
    if (typeof mdx === 'string') {
      const parseStr = native.parse || native.parseToJson || native.parse_to_json;
      if (!parseStr) throw new Error("Native string parser missing.");
      result = parseStr(mdx);
    } else {
      const buf = normalizeToBuffer(mdx);
      const parseBin = native.parseToBinary || native.parse_to_binary;
      
      if (parseBin) {
        result = parseBin(buf);
      } else {
        const parseStr = native.parse || native.parseToJson || native.parse_to_json;
        result = parseStr(buf.toString('utf-8'));
      }
    }
  } catch (err: any) {
    const sourceSnippet = typeof mdx === 'string' ? mdx.slice(0, 50) : "Binary Data";
    throw new MDXParseError(err?.message ?? String(err), sourceSnippet);
  }
  

  let parsedAst: AstNode[] = [];

  if (result instanceof Uint8Array || Buffer.isBuffer(result)) {
    const decoder = new MdxBinaryDecoder(result);
    parsedAst = decoder.decode();
  } 
  else if (typeof result === "string") {
    try { 
      parsedAst = JSON.parse(result) as AstNode[]; 
    } catch { 
      throw new Error("Invalid JSON string."); 
    }
  } 
  else if (typeof result === "object" && result !== null) {
    if (typeof result.toJson === "function") {
      try {
        const jsonString = result.toJson();
        parsedAst = JSON.parse(jsonString) as AstNode[];
      } catch (e) {
        throw new Error("[toaq-oss/omni-mdx] Failed to serialize MdxAst to JSON.");
      }
    } 
    else if (typeof result.to_json === "function") {
      try { 
        parsedAst = JSON.parse(result.to_json()) as AstNode[]; 
      } catch (e) {}
    } 
    else if (Array.isArray(result)) {
      parsedAst = result as AstNode[];
    }
  }

  if (!parsedAst || parsedAst.length === 0 && !Array.isArray(parsedAst)) {
    throw new Error("[toaq-oss/omni-mdx] Unrecognized return format from Rust parser. Available properties: " + Object.keys(result.__proto__ || result).join(", "));
  }

  if (options?.rehypePlugins && options.rehypePlugins.length > 0) {
    return await runUnifiedPipeline(parsedAst, options.rehypePlugins);
  }

  return parsedAst;
}

/**
 * Synchronously parses raw MDX content into an Abstract Syntax Tree (AST).
 * @param mdx - The raw MDX content as a string, or a binary buffer for optimal performance.
 * @param options - Optional configuration to inject `unified` plugins. 
 * **Warning:** Any plugins provided here MUST be entirely synchronous.
 * @returns An array of `AstNode` representing the parsed MDX.
 * @throws {MDXParseError} If the MDX syntax is invalid or the underlying native parser fails.
 */
export function parseMdxSync(mdx: MdxInput, options?: OmniMdxOptions): AstNode[] {
  const native = getNativeModuleSync();
  let result: any;

  try {
    if (typeof mdx === 'string') {
      const parseStr = native.parse || native.parseToJson || native.parse_to_json;
      result = parseStr(mdx);
    } else {
      const buf = normalizeToBuffer(mdx);
      const parseBin = native.parseToBinary || native.parse_to_binary;
      if (parseBin) {
        result = parseBin(buf);
      } else {
        const parseStr = native.parse || native.parseToJson || native.parse_to_json;
        result = parseStr(buf.toString('utf-8'));
      }
    }
  } catch (err: any) {
    const sourceSnippet = typeof mdx === 'string' ? mdx.slice(0, 50) : "Binary Data";
    throw new MDXParseError(err?.message ?? String(err), sourceSnippet);
  }

  let parsedAst: AstNode[] = [];

  if (result instanceof Uint8Array || Buffer.isBuffer(result)) {
    const decoder = new MdxBinaryDecoder(result);
    parsedAst = decoder.decode();
  } 
  else if (typeof result === "string") {
    try { parsedAst = JSON.parse(result) as AstNode[]; } catch { throw new Error("Invalid JSON"); }
  } 
  else if (typeof result === "object" && result !== null) {
    if (typeof result.toJson === "function") {
      try { parsedAst = JSON.parse(result.toJson()) as AstNode[]; } catch { throw new Error("Invalid JSON"); }
    } 
    else if (typeof result.to_json === "function") {
      try { parsedAst = JSON.parse(result.to_json()) as AstNode[]; } catch { throw new Error("Invalid JSON"); }
    }
    else if (Array.isArray(result)) {
      parsedAst = result as AstNode[];
    }
  }

  if (!parsedAst || (parsedAst.length === 0 && !Array.isArray(parsedAst))) {
    throw new Error("[toaq-oss/omni-mdx] Unrecognized return format from Rust parser.");
  }

  if (options?.rehypePlugins && options.rehypePlugins.length > 0) {
    return runUnifiedPipelineSync(parsedAst, options.rehypePlugins);
  }

  return parsedAst;
}

/**
 * Asynchronously compiles MDX directly into a raw JSX string using the Rust core.
 * Note: This bypasses the React AST rendering step entirely.
 * @param mdx - The raw MDX content to compile.
 * @returns A promise that resolves to the compiled JSX string.
 * @throws {Error} If the underlying native module version does not support JSX compilation.
 * @throws {MDXParseError} If the MDX syntax is invalid.
 */
export async function compileToJsx(mdx: MdxInput): Promise<string> {
  const native = getNativeModuleSync();
  const compileFn = native.compileToJsx || native.compile_to_jsx;
  
  if (typeof compileFn !== "function") {
    throw new Error("[toaq-oss/omni-mdx] compileToJsx is not supported by this native module version.");
  }
  
  try {
    const buf = normalizeToBuffer(mdx);
    return compileFn(buf);
  } catch (err: any) {
    throw new MDXParseError(err?.message ?? String(err), "Binary Data");
  }
}

/**
 * Synchronously compiles MDX directly into a raw JSX string using the Rust core.
 * Note: This bypasses the React AST rendering step entirely.
 * @param mdx - The raw MDX content to compile.
 * @returns The compiled JSX string.
 * @throws {Error} If the underlying native module version does not support JSX compilation.
 * @throws {MDXParseError} If the MDX syntax is invalid.
 */
export function compileToJsxSync(mdx: MdxInput): string {
  const native = getNativeModuleSync();
  const compileFn = native.compileToJsx || native.compile_to_jsx;
  
  if (typeof compileFn !== "function") {
    throw new Error("[toaq-oss/omni-mdx] compileToJsx is not supported by this native module version.");
  }
  
  try {
    const buf = normalizeToBuffer(mdx);
    return compileFn(buf);
  } catch (err: any) {
    throw new MDXParseError(err?.message ?? String(err), "Binary Data");
  }
}

/**
 * Custom error class thrown when the Omni-Core parser encounters an invalid MDX syntax
 * or malformed binary input data.
 */
export class MDXParseError extends Error {
  /** The snippet of source code (or binary indicator) where the error occurred. */
  readonly source: string;
  
  constructor(message: string, source: string) {
    super(`MDX parse error: ${message}`);
    this.name   = "MDXParseError";
    this.source = source;
  }
}