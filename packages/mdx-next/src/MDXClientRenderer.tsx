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

import React, { ReactNode, useEffect, useRef, useState } from "react";
import katex from "katex";
import { MDXErrorBoundary } from "./MDXErrorBoundary";
import type { AstNode, MDXComponents } from "./MDXServerRenderer";



// Attr resolver (client version handles expressions fully)

type AttrValueKind =
  | { kind: "text";       value: string }
  | { kind: "expression"; value: string }
  | { kind: "boolean" }
  | { kind: "ast";        value: AstNode[] };

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
      try { return new Function(`return (${raw})`)(); } catch {}
      return raw;
    }
    case "ast":
      return <MDXClientContent ast={attr.value} components={components} />;
    default:
      return undefined;
  }
}

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
  if (node.node_type === "InlineMath") {
    const formula = extractText(node);
    try {
      const html = katex.renderToString(formula, { displayMode: false, throwOnError: false, output: "html" });
      return <span key={key} className="math math-inline" dangerouslySetInnerHTML={{ __html: html }} />;
    } catch {
      return <span key={key} className="math math-inline">{formula}</span>;
    }
  }
  if (node.node_type === "BlockMath") {
    const formula = extractText(node);
    try {
      const html = katex.renderToString(formula, { displayMode: true, throwOnError: false, output: "html" });
      return <div key={key} className="math math-display" dangerouslySetInnerHTML={{ __html: html }} />;
    } catch {
      return <div key={key} className="math math-display">{formula}</div>;
    }
  }

  const resolvedProps: Record<string, any> = {};
  if (node.attributes) {
    // 1. Sécurisation : si c'est une string (depuis N-API Rust), on la parse
    const attrs = typeof node.attributes === "string" 
      ? JSON.parse(node.attributes) 
      : node.attributes;

    // 2. Itération sur l'objet parsé
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

function MDXClientContent({
  ast,
  components,
}: {
  ast: AstNode[];
  components: MDXComponents;
}) {
  return <>{ast.map((node, i) => renderNode(node, i, components))}</>;
}

interface MDXClientRendererProps {
  /** AST from parseMdx() — must be JSON-serialisable (pass via Server Component). */
  ast: AstNode[];
  /** Component registry — same shape as MDX_COMPONENTS. */
  components?: MDXComponents;
}

export function MDXClientRenderer({
  ast,
  components = {},
}: MDXClientRendererProps) {
  const rootRef = useRef<HTMLDivElement>(null);

  if (!ast || !Array.isArray(ast)) return null;

  return (
    <div className="omni-mdx-root">
      <MDXClientContent ast={ast} components={components} />
    </div>
  );
}