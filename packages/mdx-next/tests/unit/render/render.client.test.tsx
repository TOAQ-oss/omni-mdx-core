import { describe, it, expect } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { MDXClientRenderer } from '../../../src/MDXClientRenderer';
import type { AstNode } from '../../../src/types/MdxAST';

describe('HTML Rendering – MDXClientRenderer', () => {
  
  it('renders a basic heading (h1) to standard HTML', () => {
    const mockAst: AstNode[] = [
      { 
        node_type: 'h1', 
        children: [{ content: 'Hello World', node_type: 'text' }] 
      }
    ];

    const html = renderToStaticMarkup(<MDXClientRenderer ast={mockAst} />);
    
    expect(html).toMatch(/<h1[^>]*>Hello World<\/h1>/);
  });

  it('injects custom React components successfully', () => {
    const mockAst: AstNode[] = [
      { 
        node_type: 'p', 
        children: [{ content: 'Custom Paragraph', node_type: 'text' }] 
      }
    ];

    const customComponents = {
      p: (props: any) => <p className="text-red-500" {...props} />
    };

    const html = renderToStaticMarkup(
      <MDXClientRenderer ast={mockAst} components={customComponents} />
    );

    expect(html).toContain('class="text-red-500"');
    expect(html).toContain('Custom Paragraph');
  });

  it('renders deeply nested inline elements', () => {
    const mockAst: AstNode[] = [
      { 
        node_type: 'p', 
        children: [
          { content: 'This is ', node_type: 'text' },
          { 
            node_type: 'strong', 
            children: [{ content: 'bold', node_type: 'text' }] 
          },
          { content: ' text.', node_type: 'text' }
        ] 
      }
    ];

    const html = renderToStaticMarkup(<MDXClientRenderer ast={mockAst} />);
    
    expect(html).toMatch(/<p[^>]*>This is <strong>bold<\/strong> text.<\/p>/);
  });

  it('renders inline math using KaTeX', () => {
    const mockAst: AstNode[] = [
      {
        node_type: 'p',
        children: [
          { node_type: 'text', content: 'La formule est ' },
          {
            node_type: 'InlineMath',
            attributes: { 'data-math': { kind: 'text', value: 'E=mc^2' } },
            self_closing: true
          },
          { node_type: 'text', content: '.' }
        ]
      }
    ];

    const html = renderToStaticMarkup(<MDXClientRenderer ast={mockAst} />);
    
    expect(html).toContain('katex-html');
    expect(html).toContain('>E<');
    expect(html).toContain('>=</');
    expect(html).toContain('>m<');
    expect(html).toContain('>c<');
    expect(html).toContain('>2<');
  });

  it('renders block math using KaTeX display mode', () => {
    const mockAst: AstNode[] = [
      { 
        node_type: 'BlockMath', 
        attributes: { 'data-math': { kind: 'text', value: '\frac{1}{2}' } },
        self_closing: true
      } 
    ];

    const html = renderToStaticMarkup(<MDXClientRenderer ast={mockAst} />);
    
    expect(html).toContain('katex');
  });

  it('handles unknown components and malformed attribute JSON', () => {
  const mockAst: AstNode[] = [
    { 
      node_type: 'UnknownWidget', 
      attributes: '{ invalid json }',
      children: [{ node_type: 'text', content: 'Fallback' }] 
    }
  ];
  
  const html = renderToStaticMarkup(<MDXClientRenderer ast={mockAst} />);
  
  expect(html).toContain('data-missing-component="UnknownWidget"');
  expect(html).toContain('Fallback');
});
});