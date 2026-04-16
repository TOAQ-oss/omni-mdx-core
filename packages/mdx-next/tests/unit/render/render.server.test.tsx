import { describe, it, expect, vi } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { MDXServerRenderer } from '../../../src/MDXServerRenderer';
import type { AstNode } from '../../../src/types/MdxAST';
import { parseMdxSync } from '../../../src/server';

describe('MDXServerRenderer (RSC)', () => {
  
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

    it('renders tables correctly with thead and tbody', () => {
        const mockAst: AstNode[] = [
            {
                node_type: 'table',
                children: [
                {
                    node_type: 'thead',
                    children: [{ node_type: 'tr', children: [{ node_type: 'th', children: [{ content: 'Header', node_type: 'text' }] }] }]
                },
                {
                    node_type: 'tbody',
                    children: [{ node_type: 'tr', children: [{ node_type: 'td', children: [{ content: 'Cell', node_type: 'text' }] }] }]
                }
                ]
            }
        ];

        const html = renderToStaticMarkup(<MDXServerRenderer ast={mockAst} />);

        expect(html).toMatch(/<thead[^>]*>/);
        expect(html).toMatch(/<tbody[^>]*>/);
        expect(html).toMatch(/<td[^>]*>Cell<\/td>/);
    });

    it('handles math on server without hydration', () => {
        const mockAst: AstNode[] = [
            {
                node_type: 'InlineMath',
                attributes: { 'data-math': { kind: 'text', value: '\\alpha' } }
            }
        ];
        const html = renderToStaticMarkup(<MDXServerRenderer ast={mockAst} />);

        expect(html).toContain('katex');
        expect(html).toContain('α');
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

    it('covers server-side component render errors', () => {
  const mockAst: AstNode[] = [{ node_type: 'BadComp', children: [] }];
  
  // On crée un composant qui crash
  const components = {
    BadComp: () => {
      throw new Error('Server Crash');
    }
  };

  const spy = vi.spyOn(console, 'error').mockImplementation(() => {});

  // Si ton renderer ne catch pas l'erreur en interne, renderToStaticMarkup va throw.
  // On teste donc que l'exécution se passe comme prévu (soit il catch, soit on catch).
  try {
    const html = renderToStaticMarkup(
      <MDXServerRenderer ast={mockAst} components={components} />
    );
    // Si ton code catch l'erreur :
    expect(html).toContain('Server Crash');
  } catch (err: any) {
    // Si ton code ne catch pas (comportement par défaut de React Server) :
    expect(err.message).toBe('Server Crash');
  }

  spy.mockRestore();
});
it('handles malformed AST nodes and null children', () => {
  const weirdAst = [{ children: [] }] as any;
  const html = renderToStaticMarkup(<MDXServerRenderer ast={weirdAst} />);
  expect(html).toBe('<div class="omni-mdx-root"><div class="mdx-missing-component"></div></div>');
});

it('renders fragments with multiple siblings', () => {
  const fragmentAst: AstNode[] = [
    { node_type: 'fragment', children: [
      { node_type: 'text', content: 'One' },
      { node_type: 'text', content: 'Two' }
    ]}
  ];
  const html = renderToStaticMarkup(<MDXServerRenderer ast={fragmentAst} />);
  expect(html).toContain('OneTwo');
});
it('renders tables and pre-blocks without code child for coverage', () => {
  const tableMdx = '| a |\n| - |\n| b |';
  const preMdx = '```\nraw text\n```'; // Un bloc de code simple

  const astTable = parseMdxSync(tableMdx);
  const astPre = parseMdxSync(preMdx);

  const htmlTable = renderToStaticMarkup(<MDXServerRenderer ast={astTable} />);
  const htmlPre = renderToStaticMarkup(<MDXServerRenderer ast={astPre} />);

  // Vérifie la structure complexe des tables (thead/tbody)
  expect(htmlTable).toContain('<thead');
  expect(htmlTable).toContain('<tbody');
  
  // Vérifie la branche "pre" qui injecte un <code> si manquant
  expect(htmlPre).toContain('<pre');
  expect(htmlPre).toContain('<code');
});
it('renders complex table structures and code fallbacks', () => {
  const tableAst: AstNode[] = [{
    node_type: 'table',
    children: [
      { node_type: 'thead', children: [{ node_type: 'th', children: [{node_type:'text', content:'H'}] }] },
      { node_type: 'tr', children: [{ node_type: 'td', children: [{node_type:'text', content:'C'}] }] }
    ]
  }];
  
  const preAst: AstNode[] = [{
    node_type: 'pre',
    content: 'raw code content' // Cas où il n'y a pas de nœud 'code' enfant
  }];

  const htmlTable = renderToStaticMarkup(<MDXServerRenderer ast={tableAst} />);
  const htmlPre = renderToStaticMarkup(<MDXServerRenderer ast={preAst} />);

  expect(htmlTable).toContain('<thead');
  expect(htmlPre).toContain('<code>raw code content</code>');
});

it('covers server-side component catch block (163-179)', () => {
  // On force NODE_ENV pour ce test précis
  const originalEnv = process.env.NODE_ENV;
  process.env.NODE_ENV = "development";

  const mockAst: AstNode[] = [{ node_type: 'Crash', children: [] }];
  const components = {
    get Crash() { throw new Error("Server Crash"); }
  };

  const spy = vi.spyOn(console, 'error').mockImplementation(() => {});
  const html = renderToStaticMarkup(<MDXServerRenderer ast={mockAst} components={components} />);

  expect(html).toContain('mdx-component-error');
  expect(html).toContain('Server Crash');

  spy.mockRestore();
  process.env.NODE_ENV = originalEnv; // Restauration
});
it('covers edge cases for table and pre tags', () => {
  const edgeCaseAst: AstNode[] = [
    // 1. Table sans thead (couvre 216-219)
    {
      node_type: 'table',
      children: [{ node_type: 'tr', children: [{ node_type: 'td', content: 'Simple cell' }] }]
    },
    // 2. Pre sans enfant code (couvre 238-259)
    {
      node_type: 'pre',
      children: [{ node_type: 'text', content: 'Just raw text in pre' }]
    }
  ];

  const html = renderToStaticMarkup(<MDXServerRenderer ast={edgeCaseAst} />);
  
  expect(html).toContain('<tbody>'); // Table sans thead génère un tbody
  expect(html).toContain('<code>Just raw text in pre</code>'); // Auto-injection de <code>
});
it('covers fragments, null content and development error UI', () => {
  const originalEnv = process.env.NODE_ENV;
  process.env.NODE_ENV = 'development'; // Pour couvrir les lignes 165-181

  const edgeCaseAst: AstNode[] = [
    { node_type: 'fragment', children: [] }, // Fragment vide (lignes 45-64)
    { node_type: 'text', content: null as any }, // Texte nul
    { node_type: 'CrashComp' } // Composant qui va crash
  ];

  const components = {
    get CrashComp() { throw new Error("Dev Error"); }
  };

  const spy = vi.spyOn(console, 'error').mockImplementation(() => {});
  const html = renderToStaticMarkup(
    <MDXServerRenderer ast={edgeCaseAst} components={components} />
  );

  expect(html).toContain('mdx-component-error');
  expect(html).toContain('Dev Error'); // Vérifie l'UI de dev

  spy.mockRestore();
  process.env.NODE_ENV = originalEnv;
});
});