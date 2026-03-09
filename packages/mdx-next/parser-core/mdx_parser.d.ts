/* tslint:disable */
/* eslint-disable */

/**
 * Compiles the raw MDX input into a minified JSON string representing the AST.
 *
 * This function is exposed directly to JavaScript via `wasm-bindgen`.
 * It returns a standard `JsError` if the parsing fails, ensuring that Node.js
 * or the browser can `try/catch` the error cleanly instead of encountering a WASM panic.
 */
export function parse_mdx_to_json(input: string): string;

/**
 * Compiles the raw MDX input into a pretty-printed JSON string.
 *
 * Useful for debugging, logging, or local development environments where
 * human-readable AST introspection is required.
 */
export function parse_mdx_to_json_pretty(input: string): string;

/**
 * Utility function exposed to JavaScript to fetch the compiled parser's version,
 * allowing the frontend package to verify WASM binary compatibility.
 */
export function parse_mdx_version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly mdx_free: (a: number) => void;
    readonly mdx_parse: (a: number) => number;
    readonly parse_mdx_to_json: (a: number, b: number) => [number, number, number, number];
    readonly parse_mdx_to_json_pretty: (a: number, b: number) => [number, number, number, number];
    readonly parse_mdx_version: () => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
