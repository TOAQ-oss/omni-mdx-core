"use client";

/**
 * MDXClientRenderer.tsx
 *
 * Client Component — handles KaTeX hydration and interactive components.
 *
 * You generally do NOT use this directly.
 * Use <MDXServerRenderer> in Server Components and only register Client
 * Components (wrapped in "use client") for interactive parts (charts, tabs, etc.).
 *
 * Use this component ONLY when:
 *   1. You are in a page/layout that is entirely client-side (no RSC)
 *   2. You need live re-rendering (e.g. a live MDX editor/preview)
 *
 * For the live editor use case, import from '@toaq/omni-mdx/client'.
 */

import React, { ReactNode } from "react";
import katex from "katex";
import JSON5 from 'json5';
import { MDXErrorBoundary } from "./MDXErrorBoundary";
import type { AstNode, MDXComponents } from "./types/MdxAST";
import { BASIC_STYLES } from "./utils/basicStyles";


/**
 * Represents the different types of attribute values that can come from the Rust AST.
 */
type AttrValueKind =
  | { kind: "text";       value: string }
  | { kind: "expression"; value: string }
  | { kind: "boolean" }
  | { kind: "ast";        value: AstNode[] };

/**
 * Converts a raw `AttrValueKind` from the Rust AST into a React-usable prop value on the client.
 * - `text` evaluates to a string.
 * - `boolean` evaluates to `true`.
 * - `expression` attempts to parse as JSON or a JS expression (evaluated via `new Function`).
 * - `ast` recursively renders nested MDX elements via `<MDXClientContent>`.
 *
 * @param attr - The attribute node from the AST.
 * @param components - The component registry to pass down to nested AST nodes.
 * @returns The resolved React node, string, or boolean.
 */
function resolveAttr(
  attr: AttrValueKind,
  components: MDXComponents,
): React.ReactNode | string | boolean {
  switch (attr.kind) {
    case "text":    return attr.value;
    case "boolean": return true;
    case "expression": {
      const raw = attr.value.trim();
      try { return JSON.parse(raw); } catch {}
      try { return JSON5.parse(raw); } catch {}
      return raw;
    }
    case "ast":
      return <MDXClientContent ast={attr.value} components={components} />;
    default:
      return undefined;
  }
}

/**
 * Recursively extracts plain text content from an AST node and its children.
 * Useful for extracting raw strings required for KaTeX math rendering.
 *
 * @param node - The AST node to extract text from.
 * @returns The concatenated plain text string.
 */
function extractText(node: AstNode): string {
  if (node.node_type === "text") return node.content ?? "";
  if (node.content) return node.content;
  return (node.children ?? []).map(extractText).join("");
}

const HTML_TAGS = new Set([
  "a","abbr","article","aside","b","blockquote","br","caption","cite","code",
  "col","colgroup","dd","del","details","dfn","div","dl","dt","em","figcaption",
  "figure","footer","h1","h2","h3","h4","h5","h6","header","hr","i","img","ins",
  "kbd","li","main","mark","nav","ol","p","pre","q","s","section","small","span",
  "strong","sub","summary","sup","table","tbody","td","tfoot","th","thead","tr",
  "u","ul","var",
]);

/**
 * Recursively renders a single AST node into a React element on the client side.
 * Handles text nodes, fragments, math blocks (KaTeX), HTML tags, and custom components.
 * Custom components are automatically wrapped in an `<MDXErrorBoundary>`.
 *
 * @param node - The current AST node to render.
 * @param index - The index of the node within its parent's children array (used for React keys).
 * @param components - The registry of custom MDX components.
 * @returns The constructed React node.
 */
function renderNode(
  node: AstNode,
  index: number,
  components: MDXComponents,
): ReactNode {
  const key = `${node.node_type}-${index}`;

  if (node.node_type === "text") return node.content ?? null;

  if (node.node_type === "fragment") {
    return (
      <React.Fragment key={key}>
        {node.children?.map((c, i) => renderNode(c, i, components))}
      </React.Fragment>
    );
  }

  // Math — rendered directly via KaTeX (static import)
  const getMathFormula = (node: AstNode) => {
    let formula = "";
    if (node.attributes) {
      const attrs = typeof node.attributes === "string" 
        ? JSON.parse(node.attributes) 
        : node.attributes;
        
      const mathData = attrs?.["data-math"];
      if (mathData && mathData.value) {
        formula = String(mathData.value);
      }
    }
    if (!formula) {
      formula = extractText(node);
    }
    return formula;
  };

  if (node.node_type === "InlineMath") {
    const formula = getMathFormula(node);

    try {
      const html = katex.renderToString(formula, {
        displayMode:  false,
        throwOnError: false,
        output:       "html",
      });
      return (
        <span
          key={key}
          className="math math-inline"
          dangerouslySetInnerHTML={{ __html: html }}
        />
      );
    } catch {
      return <span key={key} className="math math-inline">{formula}</span>;
    }
  }

  if (node.node_type === "BlockMath" || node.node_type === "math") {
    const formula = getMathFormula(node);

    try {
      const html = katex.renderToString(formula, {
        displayMode:  true,
        throwOnError: false,
        output:       "html",
      });
      return (
        <div
          key={key}
          className="math math-display"
          dangerouslySetInnerHTML={{ __html: html }}
        />
      );
    } catch {
      return <div key={key} className="math math-display">{formula}</div>;
    }
  }

  const resolvedProps: Record<string, any> = {};
  if (node.attributes) {
    let attrs: Record<string, unknown> = {};
    
    if (typeof node.attributes === "string") {
      try {
        attrs = JSON.parse(node.attributes) as Record<string, unknown>;
      } catch (error) {
        if (process.env.NODE_ENV === "development") {
          console.warn(
            `[toaq-oss/omni-mdx] Failed to parse attributes JSON for <${node.node_type}>`,
            error
          );
        }
      }
    } else {
      attrs = node.attributes as Record<string, unknown>;
    }

    for (const [k, v] of Object.entries(attrs)) {
      resolvedProps[k] = resolveAttr(v as AttrValueKind, components);
    }
  }

  const renderedChildren = node.children?.length
    ? node.children.map((c, i) => renderNode(c, i, components))
    : (node.content ?? undefined);

  const Custom = components[node.node_type];
  if (Custom) {
    return (
      <MDXErrorBoundary key={key} componentName={node.node_type}>
        <Custom {...resolvedProps}>
          {renderedChildren}
        </Custom>
      </MDXErrorBoundary>
    );
  }

  if (HTML_TAGS.has(node.node_type)) {
    const Tag = node.node_type as any;
    return <Tag key={key} {...resolvedProps}>{renderedChildren}</Tag>;
  }

  if (process.env.NODE_ENV === "development") {
    console.warn(`[toaq-oss/omni-mdx] Unknown component: <${node.node_type}>`);
  }
  return (
    <div key={key} data-missing-component={node.node_type} className="mdx-missing-component">
      {renderedChildren}
    </div>
  );
}

/**
 * A helper component that maps over an array of AST nodes and renders them.
 *
 * @param props.ast - The array of AST nodes to render.
 * @param props.components - The custom component registry.
 */
function MDXClientContent({
  ast,
  components,
}: {
  ast: AstNode[];
  components: MDXComponents;
}) {
  return <>{ast.map((node, i) => renderNode(node, i, components))}</>;
}

/**
 * Properties for the MDXClientRenderer component.
 */
interface MDXClientRendererProps {
  /** AST from parseMdx() — must be JSON-serialisable (pass via Server Component). */
  ast: AstNode[];
  /** Component registry — same shape as MDX_COMPONENTS. */
  components?: MDXComponents;
}

/**
 * A React Client Component that renders an Omni-MDX AST into an interactive React tree.
 * Designed specifically for client-side environments (e.g., live MDX editors).
 * @param props - Component properties including the parsed `ast` and an optional `components` mapping.
 * @returns The rendered React element tree, or `null` if the AST is invalid.
 * @example
 * ```tsx
 * import { MDXClientRenderer, parseMdxClient } from '@toaq-oss/omni-mdx/client';
 * import { useState, useEffect } from 'react';
 * 
 * export function LiveEditor({ rawMdx }) {
 *  const [ast, setAst] = useState([]);
 * 
 *  useEffect(() => {
 *    parseMdxClient(rawMdx).then(setAst);
 *  }, [rawMdx]);
 * 
 *  return <MDXClientRenderer ast={ast} />;
 * }
 * ```
 */
export function MDXClientRenderer({
  ast,
  components = {},
}: MDXClientRendererProps) {
  if (!ast || !Array.isArray(ast)) return null;

  const finalComponents = {
    ...BASIC_STYLES,
    ...components
  };

  return (
    <div className="omni-mdx-root">
      <MDXClientContent ast={ast} components={finalComponents} />
    </div>
  );
}