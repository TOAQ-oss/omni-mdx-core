/**
 * @file index.ts
 * @description Main entry point for the Omni MDX React/Next.js package.
 * Exports the core WASM parser alongside the React rendering ecosystem.
 */

export {
    parse_mdx_to_json,
    parse_mdx_to_json_pretty,
    parse_mdx_version
} from '../omni-core/mdx_parser';

export { MDXViewer } from './MDXRenderer';
export { MdxConfigProvider, useMdxConfig } from './MDXConfigContext';
export { MDXErrorBoundary } from './MDXErrorBoundary';
export { parseProps } from './utils/parserHelpers';
export type { AstNode, AttrValue } from './types';