// parse.client.ts
//
// Client-side MDX parser using WASM (--target web build).
// Designed for live editors and fully client-side rendering contexts.
// Never runs on the server — returns [] immediately if called server-side.

import type { AstNode } from "./types/MdxAST";
import { MdxBinaryDecoder } from "./utils/binaryDecoder";

let initPromise: Promise<(mdx: Uint8Array) => Uint8Array> | null = null;

function getClientParser(): Promise<(mdx: Uint8Array) => Uint8Array> {
  if (!initPromise) {
    initPromise = (async () => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const wasm: any = await import("../wasm/omni_mdx_core.js");

      if (typeof wasm.default === "function") {
        const wasmUrl = new URL("./omni_mdx_core_bg.wasm", import.meta.url);
        await wasm.default(wasmUrl);
      }

      return (mdx: Uint8Array): Uint8Array => wasm.parse_to_binary(mdx);
    })();
  }
  return initPromise;
}

function normalizeToUint8Array(input: any): Uint8Array {
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

export async function parseMdxClient(mdx: string | Uint8Array): Promise<AstNode[]> {
  if (typeof window === "undefined") return [];

  try {
    const parse = await getClientParser();
    const inputBuffer = normalizeToUint8Array(mdx);
    const binaryAst = parse(inputBuffer);
    const decoder = new MdxBinaryDecoder(binaryAst);
    return decoder.decode();
  } catch (err) {
    const errorMessage = err instanceof Error ? err.message : String(err);
    
    throw new Error(`Syntax error in MDX: ${errorMessage}`);
  }
}