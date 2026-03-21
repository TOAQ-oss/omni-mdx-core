// parse.client.ts
//
// Client-side MDX parser using WASM (--target web build).
// Designed for live editors and fully client-side rendering contexts.
// Never runs on the server — returns [] immediately if called server-side.

import type { AstNode } from "./MDXServerRenderer";

let _parseClient: ((mdx: string) => string) | null = null;

async function getClientParser(): Promise<(mdx: string) => string> {
  if (_parseClient) return _parseClient;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const wasm: any = await import("../wasm/omni_mdx_core.js");

  if (typeof wasm.default === "function") {
    const wasmUrl = new URL("./omni_mdx_core_bg.wasm", import.meta.url);
    await wasm.default(wasmUrl);
  }

  _parseClient = (mdx: string): string => wasm.parse_mdx_to_json(mdx);
  return _parseClient!;
}

export async function parseMdxClient(mdx: string): Promise<AstNode[]> {
  if (typeof window === "undefined") return [];

  try {
    const parse = await getClientParser();
    const json  = parse(mdx);
    return JSON.parse(json) as AstNode[];
  } catch (err) {
    console.error("[omni-mdx] WASM client parse error:", err);
    return [];
  }
}