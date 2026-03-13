/**
 * @toaq/mdx-engine
 *
 * Safe entry point — types only; no server/native/wasm imports.
 * To parse and render, use:
 *   import { parseMdx, MDXServerRenderer } from '@toaq/mdx-engine/server'
 *   import { MDXClientRenderer }           from '@toaq/mdx-engine/client'
 */

import React from "react";

// Types only (no runtime imports)
export type { AstNode, AttrValueKind, MDXComponents } from "./MDXServerRenderer";

// Component registry (empty by default; to be expanded)
export const MDX_COMPONENTS: Record<string, React.ComponentType<any>> = {};

// Utility for parsing props in custom components
export { parseProps } from "./utils/parserHelpers";