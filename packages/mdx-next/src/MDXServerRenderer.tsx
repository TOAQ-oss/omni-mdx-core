/**
 * MDXServerRenderer.tsx
 *
 * React Server Component — renders an MDX AST on the server.
 * No "use client" — safe for Next.js SSR, SSG, and ISR.
 *
 * Drop-in replacement for <MDXRemote source={...} components={MDX_COMPONENTS} />
 *
 * Usage in a Server Component (page.tsx):
 *
 *   import { MDXServerRenderer, parseMdx } from '@toaq/omni-mdx/server';
 *   import { MDX_COMPONENTS } from './MDXComponents';
 *
 *   export default async function Page() {
 *     const ast = await parseMdx(mdxString);
 *     return <MDXServerRenderer ast={ast} components={MDX_COMPONENTS} />;
 *   }
 */

import React, { ReactNode, JSX } from "react";
import katex from "katex";

// AST types (mirror Rust output exactly)

export type AttrValueKind =
  | { kind: "text";       value: string }
  | { kind: "expression"; value: string }
  | { kind: "boolean" }
  | { kind: "ast";        value: AstNode[] };

export interface AstNode {
  node_type:     string;
  content?:      string;
  self_closing?: boolean;
  attributes?:   Record<string, AttrValueKind>;
  children?:     AstNode[];
}

export type MDXComponents = Record<string, React.ComponentType<any>>;

/**
 * Converts a raw AttrValueKind from the Rust AST into a React-usable prop value.
 *   text       → string
 *   expression → attempt JSON/JS parse, fallback to string
 *   boolean    → true
 *   ast        → <MDXServerRenderer ast={...} components={...} />
 */
function resolveAttr(
  attr: AttrValueKind,
  components: MDXComponents,
): React.ReactNode | string | boolean {
  switch (attr.kind) {
    case "text":
      return attr.value;

    case "boolean":
      return true;

    case "expression": {
      const raw = attr.value.trim();
      // Try JSON first (safe), then JS expression as last resort
      try { return JSON.parse(raw); } catch {}
      try { return new Function(`return (${raw})`)(); } catch {}
      return raw;
    }

    case "ast":
      return (
        <MDXServerRenderer
          ast={attr.value}
          components={components}
        />
      );

    default:
      return undefined;
  }
}

const HTML_TAGS = new Set([
  "a","abbr","address","article","aside","b","bdi","bdo","blockquote","br",
  "caption","cite","code","col","colgroup","data","dd","del","details","dfn",
  "div","dl","dt","em","figcaption","figure","footer","h1","h2","h3","h4",
  "h5","h6","header","hr","i","img","ins","kbd","li","main","mark","nav","ol",
  "p","pre","q","rp","rt","ruby","s","samp","section","small","span","strong",
  "sub","summary","sup","table","tbody","td","tfoot","th","thead","time","tr",
  "u","ul","var","wbr",
]);

function extractText(node: AstNode): string {
  if (node.node_type === "text") return node.content ?? "";
  if (node.content) return node.content;
  return (node.children ?? []).map(extractText).join("");
}

function renderNode(
  node: AstNode,
  index: number,
  components: MDXComponents,
): ReactNode {
  const key = `${node.node_type}-${index}`;

  if (node.node_type === "text") {
    return node.content ?? null;
  }

  if (node.node_type === "fragment") {
    return (
      <React.Fragment key={key}>
        {renderChildren(node, components)}
      </React.Fragment>
    );
  }

  // Math — rendu serveur via KaTeX (HTML statique, zéro JS client)
  if (node.node_type === "InlineMath") {
    try {
      const html = katex.renderToString(node.content ?? "", {
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
      return <span key={key} className="math math-inline">{node.content}</span>;
    }
  }

  if (node.node_type === "BlockMath") {
    try {
      const html = katex.renderToString(node.content ?? "", {
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
      return <div key={key} className="math math-display">{node.content}</div>;
    }
  }

  // Resolve props from AST attributes
  const resolvedProps: Record<string, any> = {};
  if (node.attributes) {
    for (const [k, v] of Object.entries(node.attributes)) {
      resolvedProps[k] = resolveAttr(v, components);
    }
  }

  const renderedChildren = node.children && node.children.length > 0
    ? renderChildren(node, components)
    : (node.content ?? undefined);

  // Custom registered component
  const Custom = components[node.node_type];
  if (Custom) {
    try {
      return (
        <Custom key={key} {...resolvedProps}>
          {renderedChildren}
        </Custom>
      );
    } catch (err) {
      if (process.env.NODE_ENV === "development") {
        console.error(`[toaq-oss/omni-mdx] Server render failed for <${node.node_type}>:`, err);
      }
      return (
        <div
          key={key}
          className="mdx-component-error"
          data-component={node.node_type}
          style={{ padding: "1rem", border: "2px solid #ef4444", borderRadius: "0.5rem", margin: "1rem 0", background: "#fef2f2" }}
        >
          <strong style={{ color: "#b91c1c" }}>Render error: &lt;{node.node_type}&gt;</strong>
          {process.env.NODE_ENV === "development" && (
            <pre style={{ color: "#dc2626", fontSize: "0.875rem", marginTop: "0.5rem" }}>
              {String(err)}
            </pre>
          )}
        </div>
      );
    }
  }

  if (node.node_type === "table") {
    const firstChild = node.children?.[0];
    const hasThead   = firstChild?.node_type === "thead";
    const theadNode  = hasThead ? firstChild : null;
    const bodyRows   = hasThead ? node.children!.slice(1) : node.children ?? [];

    const theadEl = theadNode ? (
      <thead key="thead">
        <tr>
          {theadNode.children?.map((cell, i) =>
            renderNode({ ...cell, node_type: "th" }, i, components)
          )}
        </tr>
      </thead>
    ) : null;

    const tbodyEl = bodyRows.length > 0 ? (
      <tbody key="tbody">
        {bodyRows.map((row, i) => renderNode(row, i, components))}
      </tbody>
    ) : null;

    return <table key={key} {...resolvedProps}>{theadEl}{tbodyEl}</table>;
  }

  if (node.node_type === "pre") {
    const rawText = extractText(node);
    return <pre key={key} {...resolvedProps}><code>{rawText}</code></pre>;
  }

  if (HTML_TAGS.has(node.node_type)) {
    const Tag = node.node_type as keyof JSX.IntrinsicElements;
    return (
      <Tag key={key} {...resolvedProps}>
        {renderedChildren}
      </Tag>
    );
  }

  if (process.env.NODE_ENV === "development") {
    console.warn(
      `[toaq-oss/omni-mdx]Unknown component: <${node.node_type}>. ` +
      `Register it via MDX_COMPONENTS or add it to your components prop.`
    );
  }
  return (
    <div key={key} data-missing-component={node.node_type} className="mdx-missing-component">
      {renderedChildren}
    </div>
  );
}

function renderChildren(node: AstNode, components: MDXComponents): ReactNode[] {
  if (!node.children?.length) return [];
  return node.children.map((child, i) => renderNode(child, i, components));
}

interface MDXServerRendererProps {
  /** AST produced by parseMdx() — JSON-serialisable. */
  ast: AstNode[];
  /**
   * Component registry. Keys are JSX tag names (e.g. "Note", "Details").
   * Values are React components (Server or Client).
   *
   * Server Components render on the server (recommended for static content).
   * Client Components receive "use client" and hydrate in the browser.
   */
  components?: MDXComponents;
}

export function MDXServerRenderer({
  ast,
  components = {},
}: MDXServerRendererProps): JSX.Element {
  if (!ast || !Array.isArray(ast)) {
    return <></>;
  }

  return (
    <div className="omni-mdx-root">
      {ast.map((node, i) => renderNode(node, i, components))}
    </div>
  );
}