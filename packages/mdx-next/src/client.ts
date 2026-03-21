"use client";

/**
 * @toaq/omni-mdx/client
 *
 * Client-only entry point.
 * Use for live MDX editors, previews, or any fully client-side rendering.
 */

export { MDXClientRenderer } from "./MDXClientRenderer";
export { MDXErrorBoundary } from "./MDXErrorBoundary";
export { parseMdxClient as parseMdx } from "./parse.client";
export type { AstNode, MDXComponents } from "./MDXServerRenderer";