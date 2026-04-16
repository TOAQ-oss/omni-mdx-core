import { describe, it, expect, vi } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { MDXServerRenderer } from '../../../src/MDXServerRenderer';
import type { AstNode } from '../../../src/types/MdxAST';
import { parseMdxSync } from '../../../src/server';

describe('MDXServerRenderer (RSC)', () => {
  
    it('returns empty fragment for invalid or null AST', () => {
        const html = renderToStaticMarkup(<MDXServerRenderer ast={null as any} />);
        expect(html).toBe('');
    });

    it('renders static HTML without client-side overhead', () => {
        const mockAst: AstNode[] = [
        { node_type: 'h1', children: [{ content: 'Server Side', node_type: 'text' }] },
        { node_type: 'p', children: [{ content: 'Rendu statique.', node_type: 'text' }] }
        ];

        const html = renderToStaticMarkup(<MDXServerRenderer ast={mockAst} />);
        
        expect(html).toContain('omni-mdx-root');
        expect(html).toMatch(/<h1[^>]*>Server Side<\/h1>/);
        expect(html).toContain('Rendu statique.');
    });

    it('renders complex table structures and code blocks', () => {
        const tableMdx = '| Header |\n| ------ |\n| Cell |';
        const preMdx = '```\nraw text\n```';

        const astTable = parseMdxSync(tableMdx);
        const astPre = parseMdxSync(preMdx);

        const htmlTable = renderToStaticMarkup(<MDXServerRenderer ast={astTable} />);
        const htmlPre = renderToStaticMarkup(<MDXServerRenderer ast={astPre} />);

        expect(htmlTable).toContain('<thead');
        expect(htmlTable).toContain('<tbody');
        expect(htmlTable).toContain('<td');
        
        expect(htmlPre).toContain('<pre');
        expect(htmlPre).toContain('<code');
    });

    it('renders void HTML elements correctly (e.g., img, br)', () => {
        const mockAst: AstNode[] = [
            { node_type: 'img', attributes: { src: { kind: 'text', value: 'test.png' } } },
            { node_type: 'br' }
        ];
        const html = renderToStaticMarkup(<MDXServerRenderer ast={mockAst} />);
        
        expect(html).toContain('<img');
        expect(html).toContain('src="test.png"');
        expect(html).toContain('<br');
    });

    it('handles math on server without hydration (Inline and Block)', () => {
        const mockAst: AstNode[] = [
            {
                node_type: 'InlineMath',
                attributes: { 'data-math': { kind: 'text', value: '\\alpha' } }
            },
            {
                node_type: 'BlockMath',
                attributes: { 'data-math': { kind: 'text', value: '\\beta' } }
            }
        ];
        const html = renderToStaticMarkup(<MDXServerRenderer ast={mockAst} />);

        expect(html).toContain('katex');
        expect(html).toContain('α');
        expect(html).toContain('β');
    });

    it('covers all resolveAttr branches for server components', () => {
        const mockAst: AstNode[] = [{
            node_type: 'CustomWidget',
            attributes: {
                isTrue: { kind: 'boolean' },
                jsonExpr: { kind: 'expression', value: '{"val": 42}' },
                jsExpr: { kind: 'expression', value: '10 + 5' },
                rawExpr: { kind: 'expression', value: 'not-json' },
                nested: { kind: 'ast', value: [{ node_type: 'text', content: 'NestedAST' }] }
            }
        }];

        const components = {
            CustomWidget: (props: any) => (
                <div>
                    {props.isTrue && 'BOOL_OK '}
                    {props.jsonExpr?.val === 42 && 'JSON_OK '}
                    {props.jsExpr === 15 && 'JS_OK '}
                    {props.rawExpr === 'not-json' && 'RAW_OK '}
                    {props.nested}
                </div>
            )
        };

        const html = renderToStaticMarkup(<MDXServerRenderer ast={mockAst as any} components={components} />);
        
        expect(html).toContain('BOOL_OK');
        expect(html).toContain('JSON_OK');
        expect(html).toContain('JS_OK');
        expect(html).toContain('RAW_OK');
        expect(html).toContain('NestedAST');
    });

    it('handles unknown components and malformed attribute JSON', () => {
        const mockAst: AstNode[] = [
            { 
                node_type: 'UnknownWidget', 
                attributes: '{ invalid json }',
                children: [{ node_type: 'text', content: 'Fallback' }] 
            }
        ];
        
        const html = renderToStaticMarkup(<MDXServerRenderer ast={mockAst} />);
        
        expect(html).toContain('data-missing-component="UnknownWidget"');
        expect(html).toContain('Fallback');
    });

    it('handles malformed AST nodes and null children', () => {
        const weirdAst = [{ children: [] }] as any;
        const html = renderToStaticMarkup(<MDXServerRenderer ast={weirdAst} />);
        expect(html).toContain('mdx-missing-component');
    });

    it('covers edge cases: fragments, null content, math fallbacks, and catch blocks (Dev)', () => {
        vi.stubEnv('NODE_ENV', 'development');
        const spy = vi.spyOn(console, 'error').mockImplementation(() => {});

        const edgeCaseAst: AstNode[] = [
            { node_type: 'fragment', children: [{ node_type: 'text', content: 'FragContent' }] },
            { node_type: 'text', content: null as any },
            { node_type: 'InlineMath', children: [{ node_type: 'text', content: 'E=mc^2' }] },
            { node_type: 'math', children: [{ node_type: 'text', content: 'E=mc^2' }] },
            { node_type: 'CrashNode' }
        ];

        const components:any = {
            get CrashNode() { throw new Error("Server Error"); } 
        };

        const html = renderToStaticMarkup(<MDXServerRenderer ast={edgeCaseAst as any} components={components} />);

        expect(html).toContain('FragContent'); 
        expect(html).toContain('math-inline'); 
        expect(html).toContain('math-display'); 
        expect(html).toContain('katex');
        expect(html).toContain('mdx-component-error');
        expect(html).toContain('Server Error');

        vi.unstubAllEnvs();
        spy.mockRestore();
    });

    it('covers the catch block in production (masks the traces)', () => {
        vi.stubEnv('NODE_ENV', 'production');
        
        const edgeCaseAst: AstNode[] = [{ node_type: 'CrashNode' }];
        const components: any = {
            get CrashNode() { throw new Error("Prod Error"); } 
        };

        const html = renderToStaticMarkup(<MDXServerRenderer ast={edgeCaseAst as any} components={components} />);

        expect(html).toContain('mdx-component-error');
        expect(html).not.toContain('Prod Error');

        vi.unstubAllEnvs();
    });
});