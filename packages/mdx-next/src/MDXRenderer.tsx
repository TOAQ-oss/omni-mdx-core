"use client";

import React, { useState, useEffect, ReactNode } from 'react';

import { MdxConfigProvider } from './MDXConfigContext';
import { MDX_COMPONENTS } from './MDXComponents';
import { MDXViewerProps } from './interface/MDXEngine';
import { MDXErrorBoundary } from './MDXErrorBoundary';
import { InlineMath, BlockMath } from 'react-katex';
import 'katex/dist/katex.min.css';
import { parse_mdx_to_json } from '../wasm-core/core_parser';


interface AstNode {
    type: string;
    content?: string;
    children?: AstNode[];
}

export const MDXViewer = ({ content, imageComponent, onErrorChange }: MDXViewerProps) => {
  const [ast, setAst] = useState<AstNode[] | null>(() => {
      if (!content) return null;
      try {
          const rawJson = parse_mdx_to_json(content);
          return JSON.parse(rawJson);
      } catch (err) {
          console.error("Erreur critique du moteur Rust:", err);
          return null;
      }
  });

  useEffect(() => {
      if (!document.getElementById('katex-css')) {
          const link = document.createElement('link');
          link.id = 'katex-css';
          link.rel = 'stylesheet';
          link.href = 'https://cdn.jsdelivr.net/npm/katex@0.16.0/dist/katex.min.css';
          document.head.appendChild(link);
      }
  }, []);

  const renderNode = (node: AstNode, index: number): ReactNode => {
      if (node.type === 'text' || node.type === 'Text') {
          return <React.Fragment key={index}>{node.content}</React.Fragment>;
      }

      if (node.type === 'inlineMath') {
          return <InlineMath key={index} math={node.content || ''} />;
      }
      
      if (node.type === 'displayMath') {
          return <BlockMath key={index} math={node.content || ''} />;
      }

      const Component = (MDX_COMPONENTS as any)[node.type];

      const renderedChildren = node.children 
          ? node.children.map((child, i) => renderNode(child, i))
          : node.content;

      if (Component) {
          return <Component key={index}>{renderedChildren}</Component>;
      }

      const HtmlTag = node.type as keyof React.JSX.IntrinsicElements;
      try {
          return <HtmlTag key={index}>{renderedChildren}</HtmlTag>;
      } catch {
          return <div key={index} className="unknown-element">{renderedChildren}</div>;
      }
  };

  if (!ast) return null;

  return (
    <MdxConfigProvider ImageComponent={imageComponent}>
      <div className="mdx-content relative min-h-[100px]">
        <MDXErrorBoundary componentName="MDX Renderer">
          
          {ast.map((node, i) => renderNode(node, i))}
          
       </MDXErrorBoundary>
      </div>
    </MdxConfigProvider>
  );
};