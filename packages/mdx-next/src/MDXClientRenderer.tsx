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
 * For the live editor use case, import from '@toaq/mdx-engine/client'.
 */

import React, { ReactNode, createContext, useContext, useEffect, useRef } from "react";
import { MDXErrorBoundary } from "./MDXErrorBoundary";
import type { AstNode, MDXComponents } from "./MDXServerRenderer";

// KaTeX loader (client-only, dynamic import)

let katexLoaded = false;

async function loadKatex() {
  if (katexLoaded || typeof window === "undefined") return;
  try {
    // Dynamically import KaTeX auto-render (avoids SSR issues)
    const [{ default: katex }, autoRenderMod] = await Promise.all([
      import("katex"),
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      import("katex/contrib/auto-render") as any,
    ]);
    const renderMathInElement = autoRenderMod.default ?? autoRenderMod;
    katexLoaded = true;
    return renderMathInElement;
  } catch {
    console.warn("[toaq/mdx-engine] KaTeX not available. Install: npm install katex");
    return null;
  }
}

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

  // Math — rendered as semantic spans; KaTeX hydrates them via useEffect
  if (node.node_type === "InlineMath") {
    return (
      <span
        key={key}
        className="math math-inline"
        data-math={node.content ?? ""}
        suppressHydrationWarning
      />
    );
  }
  if (node.node_type === "BlockMath") {
    return (
      <div
        key={key}
        className="math math-display"
        data-math={node.content ?? ""}
        suppressHydrationWarning
      />
    );
  }

  const resolvedProps: Record<string, any> = {};
  if (node.attributes) {
    for (const [k, v] of Object.entries(node.attributes)) {
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
    console.warn(`[toaq/mdx-engine] Unknown component: <${node.node_type}>`);
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
  /** If true, activates KaTeX auto-render after mount. Default: true. */
  katex?: boolean;
}

export function MDXClientRenderer({
  ast,
  components = {},
  katex = true,
}: MDXClientRendererProps) {
  const rootRef = useRef<HTMLDivElement>(null);

  // Hydrate math after mount
  useEffect(() => {
    if (!katex || !rootRef.current) return;
    loadKatex().then((renderMath) => {
      if (!renderMath || !rootRef.current) return;
      renderMath(rootRef.current, {
        delimiters: [
          { left: "$$", right: "$$", display: true },
          { left: "$",  right: "$",  display: false },
        ],
      });
      // Also handle data-math attributes directly
      rootRef.current.querySelectorAll<HTMLElement>("[data-math]").forEach((el) => {
        const math = el.getAttribute("data-math") ?? "";
        const display = el.classList.contains("math-display");
        try {
          import("katex").then(({ default: k }) => {
            el.innerHTML = k.renderToString(math, { displayMode: display, throwOnError: false });
          });
        } catch {}
      });
    });
  }, [ast, katex]);

  if (!ast || !Array.isArray(ast)) return null;

  return (
    <div ref={rootRef} className="omni-mdx-root">
      <MDXClientContent ast={ast} components={components} />
    </div>
  );
}