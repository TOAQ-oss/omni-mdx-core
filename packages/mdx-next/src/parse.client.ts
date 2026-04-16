// parse.client.ts
//
// Client-side MDX parser using WASM (--target web build).
// Designed for live editors and fully client-side rendering contexts.
// Never runs on the server — returns [] immediately if called server-side.

import type { AstNode } from "./types/MdxAST";
import { MdxBinaryDecoder } from "./utils/binaryDecoder";
import { runUnifiedPipeline } from "./utils/unifiedBridge";
import type { OmniMdxOptions } from "./types/MdxInput";

type BufferLikeInput = { type: "Buffer"; data: number[] };
type NormalizableInput = string | Uint8Array | BufferLikeInput;

let initPromise: Promise<(mdx: Uint8Array) => Uint8Array> | null = null;

/**
 * Initializes and returns the WebAssembly parser function.
 * This utilizes a singleton pattern to ensure the WASM module is loaded and instantiated only once per client session.
 *
 * @returns A promise resolving to the core WASM parsing function.
 */
function getClientParser(): Promise<(mdx: Uint8Array) => Uint8Array> {
  if (!initPromise) {
    initPromise = (async () => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const wasm: any = await import("../wasm/omni_mdx_core.js");

      if (typeof wasm.default === "function") {
        const wasmUrl = new URL("./omni_mdx_core_bg.wasm", import.meta.url);
        await wasm.default({ module_or_path: wasmUrl });
      }

      return (mdx: Uint8Array): Uint8Array => wasm.parse_to_binary(mdx);
    })();
  }
  return initPromise;
}

/**
 * Converts various input formats into a standardized `Uint8Array` for WebAssembly consumption.
 *
 * @param input - The raw input data (string, Buffer object, or Uint8Array).
 * @returns A normalized `Uint8Array` ready to be parsed by the WASM core.
 */
function normalizeToUint8Array(input: NormalizableInput): Uint8Array {
  if (input instanceof Uint8Array) {
    return input;
  }
  if (typeof input === "string") {
    return new TextEncoder().encode(input);
  }
  if (input && typeof input === 'object' && input.type === 'Buffer' && Array.isArray(input.data)) {
    return new Uint8Array(input.data);
  }
  return new TextEncoder().encode(String(input));
}

/**
 * Parses MDX content directly in the browser using WebAssembly.
 * 
 * ⚠️ **Warning:** This function is strictly for client-side use. If called on the server, it immediately returns an empty array.
 *
 * Designed for live MDX editors, real-time previews, or fully client-side rendering (CSR) contexts.
 *
 * @param mdx - The source MDX code to parse. Can be a raw string or a binary `Uint8Array`.
 * @param options - Optional configuration for the unified pipeline. ⚠️ **Note:** Ensure any `rehypePlugins` provided are browser-safe (i.e., they do not rely on Node.js APIs like `fs` or `path`).
 * @returns A promise that resolves to an array of `AstNode` representing the parsed syntax tree.
 * @throws {Error} If a syntax error occurs during parsing or if the WASM module fails to process the input.
 *
 * @example
 * ```typescript
 * import { parseMdxClient } from '@toaq-oss/omni-mdx/client';
 * import rehypeHighlight from 'rehype-highlight'; // Ensure this is the browser-compatible version
 *
 * const ast = await parseMdxClient("# Hello World", {
 *  rehypePlugins: [rehypeHighlight]
 * });
 * ```
 */
export async function parseMdxClient(mdx: string | Uint8Array, options?: OmniMdxOptions): Promise<AstNode[]> {
  if (typeof window === "undefined") return [];

  let parsedAst: AstNode[] = [];
  try {
    const parse = await getClientParser();
    const inputBuffer = normalizeToUint8Array(mdx);
    const binaryAst = parse(inputBuffer);
    const decoder = new MdxBinaryDecoder(binaryAst);
    
    parsedAst = decoder.decode();
  } catch (err) {
    const errorMessage = err instanceof Error ? err.message : String(err);
    
    throw new Error(`Syntax error in MDX: ${errorMessage}`);
  }

  if (options?.rehypePlugins && options.rehypePlugins.length > 0) {
    return await runUnifiedPipeline(parsedAst, options.rehypePlugins);
  }

  return parsedAst;
}