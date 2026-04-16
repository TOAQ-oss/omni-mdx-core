import { describe, it, expect, vi } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { MDXClientRenderer } from '../../../src/MDXClientRenderer';
import type { AstNode } from '../../../src/types/MdxAST';
import katex from 'katex';

describe('HTML Rendering - MDXClientRenderer', () => {
  
  it('returns null for invalid or null AST', () => {
    const html = renderToStaticMarkup(<MDXClientRenderer ast={null as any} />);
    expect(html).toBe(''); 
  });

  it('renders fragments correctly', () => {
    const mockAst: AstNode[] = [{ 
      node_type: 'fragment', 
      children: [{ node_type: 'text', content: 'FragmentText' }] 
    }];
    const html = renderToStaticMarkup(<MDXClientRenderer ast={mockAst} />);
    expect(html).toContain('FragmentText');
  });

  it('renders a basic heading (h1) to standard HTML', () => {
    const mockAst: AstNode[] = [
      { node_type: 'h1', children: [{ content: 'Hello World', node_type: 'text' }] }
    ];
    const html = renderToStaticMarkup(<MDXClientRenderer ast={mockAst} />);
    expect(html).toMatch(/<h1[^>]*>Hello World<\/h1>/);
  });

  it('injects custom React components successfully', () => {
    const mockAst: AstNode[] = [
      { node_type: 'p', children: [{ content: 'Custom Paragraph', node_type: 'text' }] }
    ];

    const customComponents = {
      p: (props: any) => <p className="text-red-500" {...props} />
    };

    const html = renderToStaticMarkup(<MDXClientRenderer ast={mockAst} components={customComponents} />);
    expect(html).toContain('class="text-red-500"');
    expect(html).toContain('Custom Paragraph');
  });

  it('renders deeply nested inline elements', () => {
    const mockAst: AstNode[] = [
      { 
        node_type: 'p', 
        children: [
          { content: 'This is ', node_type: 'text' },
          { node_type: 'strong', children: [{ content: 'bold', node_type: 'text' }] },
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
          { node_type: 'InlineMath', attributes: { 'data-math': { kind: 'text', value: 'E=mc^2' } } }
        ]
      }
    ];
    const html = renderToStaticMarkup(<MDXClientRenderer ast={mockAst} />);
    expect(html).toContain('katex-html');
  });

  it('renders block math using KaTeX display mode', () => {
    const mockAst: AstNode[] = [
      { node_type: 'BlockMath', attributes: { 'data-math': { kind: 'text', value: '\\frac{1}{2}' } } } 
    ];
    const html = renderToStaticMarkup(<MDXClientRenderer ast={mockAst} />);
    expect(html).toContain('katex');
  });

  it('handles math fallbacks (extractText and KaTeX catch blocks)', () => {
    const spy = vi.spyOn(katex, 'renderToString').mockImplementation(() => {
        throw new Error('Fatal KaTeX Error');
    });

    const ast: AstNode[] = [
        { node_type: 'InlineMath', children: [{ node_type: 'text', content: 'a^2+b^2=c^2' }] },
        { node_type: 'BlockMath', attributes: { 'data-math': { kind: 'text', value: 'E=mc^2' } } }
    ];
    
    const html = renderToStaticMarkup(<MDXClientRenderer ast={ast as any} />);

    expect(html).toContain('a^2+b^2=c^2');
    expect(html).toContain('E=mc^2');

    spy.mockRestore();
  });

  it('resolves complex expressions (JSON5 and raw fallback)', () => {
    const components = {
       Comp: (props: any) => <div>{props.json5.key} | {props.raw}</div>
    };
    const ast = [{
        node_type: 'Comp',
        attributes: {
            json5: { kind: 'expression', value: "{ key: 'json5-ok' }" }, 
            raw: { kind: 'expression', value: "just-a-raw-string" }
        }
    }];
    const html = renderToStaticMarkup(<MDXClientRenderer ast={ast as any} components={components} />);
    
    expect(html).toContain('json5-ok');
    expect(html).toContain('just-a-raw-string');
  });

  it('covers all attribute types resolution (boolean, expression, ast)', () => {
    const components = {
      Showcase: (props: any) => (
        <div className="showcase">
          <span id="nested">{props.nestedProp}</span>
          <span id="expr">{props['data-expr']}</span>
          <span id="bool">{props['data-bool'] ? 'is-true' : 'is-false'}</span>
        </div>
      )
    };

    const ast = [{
      node_type: 'Showcase',
      attributes: {
        'data-bool': { kind: 'boolean' },
        'data-expr': { kind: 'expression', value: '42' },
        'nestedProp': { kind: 'ast', value: [{ node_type: 'text', content: 'nested-success' }] }
      }
    }];

    const html = renderToStaticMarkup(<MDXClientRenderer ast={ast as any} components={components} />);

    expect(html).toContain('nested-success');
    expect(html).toContain('42');
    expect(html).toContain('is-true');
  });

  it('logs warnings in development mode for invalid attributes and unknown components', () => {
    vi.stubEnv('NODE_ENV', 'development');
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    const ast: AstNode[] = [
        { node_type: 'UnknownThing', attributes: '{invalid-json}', children: [{ node_type: 'text', content: 'FallbackText' }] }
    ];
    
    const html = renderToStaticMarkup(<MDXClientRenderer ast={ast} />);
    
    expect(html).toContain('data-missing-component="UnknownThing"');
    expect(html).toContain('FallbackText');

    expect(warnSpy).toHaveBeenCalledTimes(2); 

    vi.unstubAllEnvs();
    warnSpy.mockRestore();
  });
});