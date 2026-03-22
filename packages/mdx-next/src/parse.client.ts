// parse.client.ts
//
// Client-side MDX parser using WASM (--target web build).
// Designed for live editors and fully client-side rendering contexts.
// Never runs on the server — returns [] immediately if called server-side.

import type { AstNode } from "./MDXServerRenderer";
import { MdxBinaryDecoder } from "./utils/binaryDecoder";

let _parseClient: ((mdx: Uint8Array) => Uint8Array) | null = null;

async function getClientParser(): Promise<(mdx: Uint8Array) => Uint8Array> {
  if (_parseClient) return _parseClient;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const wasm: any = await import("../wasm/omni_mdx_core.js");

  if (typeof wasm.default === "function") {
    const wasmUrl = new URL("./omni_mdx_core_bg.wasm", import.meta.url);
    await wasm.default(wasmUrl);
  }

  _parseClient = (mdx: Uint8Array): Uint8Array => wasm.parse_to_binary(mdx);
  return _parseClient!;
}

export async function parseMdxClient(mdx: string | Uint8Array): Promise<AstNode[]> {
  if (typeof window === "undefined") return [];

  try {
    const parse = await getClientParser();
    const inputBuffer = typeof mdx === "string" 
      ? new TextEncoder().encode(mdx) 
      : mdx;
    const binaryAst = parse(inputBuffer);
    const decoder = new MdxBinaryDecoder(binaryAst);
    return decoder.decode();
  } catch (err) {
    console.error("[omni-mdx] WASM client parse error:", err);
    return [];
  }
}