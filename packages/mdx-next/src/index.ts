/**
 * @file index.ts
 * @description Main entry point for the Omni MDX React/Next.js package.
 * Exports the core WASM parser alongside the React rendering ecosystem.
 */
import initWasm, { 
    parse_mdx_to_json as _parse_mdx_to_json,
    parse_mdx_to_json_pretty as _parse_mdx_to_json_pretty,
    parse_mdx_version as _parse_mdx_version
} from '../parser-core/mdx_parser.js';

export const parse_mdx_to_json = _parse_mdx_to_json;
export const parse_mdx_to_json_pretty = _parse_mdx_to_json_pretty;
export const parse_mdx_version = _parse_mdx_version;

const init = initWasm;
export default init;

export { MDXViewer } from './MDXRenderer.js';
export { MdxConfigProvider, useMdxConfig } from './MDXConfigContext.js';
export { MDXErrorBoundary } from './MDXErrorBoundary.js';
export { parseProps } from './utils/parserHelpers.js';
export type { AstNode, AttrValue } from './types';