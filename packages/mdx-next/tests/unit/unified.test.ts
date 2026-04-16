import { describe, it, expect } from 'vitest';
import { runUnifiedPipelineSync, runUnifiedPipeline } from '../../src/utils/unifiedBridge';
import type { AstNode } from '../../src/types/MdxAST';

describe('Unified Bridge - HAST Transformation', () => {

  it('converts basic AST to HAST and back without plugins', () => {
    const inputAst: AstNode[] = [
      { 
        node_type: 'p', 
        children: [{ node_type: 'text', content: 'Hello' }] 
      }
    ];

    const result = runUnifiedPipelineSync(inputAst, []);
    expect(result).toEqual(inputAst);
  });

  it('handles complex attributes (boolean, expression, className)', () => {
    const inputAst: AstNode[] = [
      {
        node_type: 'div',
        attributes: {
          className: { kind: 'text', value: 'bg-red-500 text-white' },
          disabled: { kind: 'boolean' },
          id: { kind: 'expression', value: 'Math.random()' }
        },
        children: []
      }
    ];

    const result = runUnifiedPipelineSync(inputAst, [
        // On passe un plugin vide juste pour déclencher le pipeline
        () => (tree) => tree 
    ]);

    expect(result[0].attributes?.className).toEqual({ kind: 'text', value: 'bg-red-500 text-white' });
    expect(result[0].attributes?.disabled).toEqual({ kind: 'boolean' });
    expect(result[0].attributes?.id).toEqual({ kind: 'text', value: 'Math.random()' }); 
  });

  it('special case: wraps <pre> content in <code> if missing', () => {
    const inputAst: AstNode[] = [
      {
        node_type: 'pre',
        attributes: { className: { kind: 'text', value: 'language-js' } },
        children: [{ node_type: 'text', content: 'const x = 1;' }]
      }
    ];

    const result = runUnifiedPipelineSync(inputAst, [() => (tree) => tree]);

    // Vérifie que toHast a injecté le <code>
    expect(result[0].node_type).toBe('pre');
    expect(result[0].children?.[0].node_type).toBe('code');
    expect(result[0].children?.[0].attributes?.className).toBeDefined();
  });

  it('handles nested AST attributes (kind: "ast")', () => {
    const nestedAst: AstNode[] = [{ node_type: 'span', children: [{ node_type: 'text', content: 'Inner' }] }];
    const inputAst: AstNode[] = [
      {
        node_type: 'CustomComp',
        attributes: {
          content: { kind: 'ast', value: nestedAst }
        }
      }
    ];

    const result = runUnifiedPipelineSync(inputAst, [() => (tree) => tree]);

    expect(result[0].attributes?.content).toMatchObject({
      kind: 'ast',
      value: nestedAst
    });
  });

  it('async: runs the async pipeline successfully', async () => {
    const inputAst: AstNode[] = [{ node_type: 'h1', children: [{ node_type: 'text', content: 'Async' }] }];
    
    // Test de la version asynchrone
    const result = await runUnifiedPipeline(inputAst, [() => async (tree) => tree]);
    
    expect(result[0].node_type).toBe('h1');
    expect(result[0].children?.[0].content).toBe('Async');
  });

  it('handles fragments correctly', () => {
      const inputAst: AstNode[] = [
          { node_type: 'fragment', children: [{ node_type: 'text', content: 'Part 1' }] }
      ];
      const result = runUnifiedPipelineSync(inputAst, [() => (tree) => tree]);
      expect(result).toEqual(inputAst);
  });
  it('covers the unified bridge ast attribute parsing', () => {
  const inputAst: AstNode[] = [{
    node_type: 'div',
    attributes: {
      nested: { kind: 'ast', value: [{ node_type: 'text', content: 'hello' }] }
    }
  }];
  
  const result = runUnifiedPipelineSync(inputAst, [() => (tree) => tree]);
  // Cette action force le passage par JSON.stringify et le parsing inverse
  expect(result[0].attributes?.nested).toBeDefined();
});
});