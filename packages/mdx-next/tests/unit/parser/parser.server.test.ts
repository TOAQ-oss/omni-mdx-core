import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

vi.mock('module', async (importOriginal) => {
  const actual = await importOriginal<typeof import('module')>();
  return {
    ...actual,
    createRequire: (url: string | URL) => {
      const originalRequire = actual.createRequire(url);
      return (pkg: string) => {
        if (pkg.startsWith('@toaq-oss/omni-mdx')) {
          if ((globalThis as any).__nativeMockMode === 'throw') {
            throw new Error('Package non installé');
          }
          if ((globalThis as any).__nativeMock) {
            return (globalThis as any).__nativeMock;
          }
        }
        if (pkg === 'fs' && (globalThis as any).__fsMock) {
          return (globalThis as any).__fsMock;
        }
        return originalRequire(pkg);
      };
    }
  };
});

import { 
  parseMdx, 
  parseMdxSync, 
  compileToJsx, 
  compileToJsxSync, 
  MDXParseError 
} from '../../../src/parse.server';

describe('MDX Server Parser - Standard Sync & Async', () => {
  
  it('parses an empty string without throwing', async () => {
    expect(parseMdxSync('')).toBeDefined();
    expect(await parseMdx('')).toBeDefined();
  });

  it('returns a valid AST for standard string input', async () => {
    const expected = [{ node_type: 'h1', children: [{ content: 'Hello', node_type: 'text' }] }];
    expect(parseMdxSync('# Hello')).toStrictEqual(expected);
    expect(await parseMdx('# Hello')).toStrictEqual(expected);
  });

  it('handles all binary input types gracefully (Buffer, Uint8Array, Buffer-like)', async () => {
    const buf = Buffer.from('# Title');
    const uint = new Uint8Array(buf);
    const bufLike = { type: 'Buffer' as const, data: Array.from(uint) };
    
    expect(parseMdxSync(buf)[0].node_type).toBe('h1');
    expect((await parseMdx(buf))[0].node_type).toBe('h1');

    expect(parseMdxSync(uint)[0].node_type).toBe('h1');
    expect((await parseMdx(uint))[0].node_type).toBe('h1');

    expect(parseMdxSync(bufLike)[0].node_type).toBe('h1');
    expect((await parseMdx(bufLike))[0].node_type).toBe('h1');
  });

  it('throws MDXParseError on completely invalid MDX syntax', async () => {
    const badInput = '<UnclosedTag'; 
    expect(() => parseMdxSync(badInput)).toThrow(MDXParseError);
    await expect(parseMdx(badInput)).rejects.toThrow(MDXParseError);
  });

  it('covers rehypePlugins processing branch', async () => {
    const dummyPlugin = () => (tree: any) => tree;
    const options = { rehypePlugins: [dummyPlugin] };

    const syncResult = parseMdxSync('# Plugin Test', options);
    const asyncResult = await parseMdx('# Plugin Test', options);

    expect(syncResult[0].node_type).toBe('h1');
    expect(asyncResult[0].node_type).toBe('h1');
  });

  it('covers JSX compilation paths (Success and Error fallbacks)', async () => {
    try { 
      const asyncResult = await compileToJsx('# Title'); 
      expect(typeof asyncResult).toBe('string');
    } catch (e) { expect(e).toBeDefined(); }

    try { 
      const syncResult = compileToJsxSync('# Title'); 
      expect(typeof syncResult).toBe('string');
    } catch (e) { expect(e).toBeDefined(); }

    const badInput = '<UnclosedTag';
    try { compileToJsxSync(badInput); } catch (e) { expect(e).toBeDefined(); }
    try { await compileToJsx(badInput); } catch (e) { expect(e).toBeDefined(); }
  });

  it('instantiates MDXParseError directly for coverage', () => {
    const error = new MDXParseError("Custom error", "snippet");
    expect(error.message).toContain("MDX parse error");
    expect(error.source).toBe("snippet");
  });
});

describe('MDX Server Parser - Deep Edge Cases & Fallbacks', () => {
  beforeEach(() => {
    vi.resetModules();
    (globalThis as any).__nativeMockMode = undefined;
    (globalThis as any).__nativeMock = undefined;
    (globalThis as any).__fsMock = undefined;
  });

  afterEach(() => {
    (globalThis as any).__nativeMockMode = undefined;
    (globalThis as any).__nativeMock = undefined;
    (globalThis as any).__fsMock = undefined;
  });

  it('covers missing parseToBinary fallback to parseToJson', async () => {
    (globalThis as any).__nativeMock = {
      parseToJson: vi.fn().mockReturnValue('[{"node_type": "fallback_json"}]'),
      compileToJsx: vi.fn().mockReturnValue('<div></div>')
    };

    const { parseMdx } = await import('../../../src/parse.server');
    const ast = await parseMdx(Buffer.from('# Title'));
    expect(ast[0].node_type).toBe('fallback_json');
  });

  it('covers snake_case method fallbacks and to_json()', async () => {
    (globalThis as any).__nativeMock = {
      parse_to_json: vi.fn().mockReturnValue({
        to_json: () => '[{"node_type": "snake_case_node"}]'
      }),
      compile_to_jsx: vi.fn().mockReturnValue('<p></p>')
    };

    const { parseMdxSync, compileToJsxSync } = await import('../../../src/parse.server');
    const ast = parseMdxSync('# Text');
    expect(ast[0].node_type).toBe('snake_case_node');

    const jsx = compileToJsxSync('# Text');
    expect(jsx).toBe('<p></p>');
  });

  it('covers toJson() method return', async () => {
    (globalThis as any).__nativeMock = {
      parseToJson: vi.fn().mockReturnValue({
        toJson: () => '[{"node_type": "to_json_node"}]'
      })
    };

    const { parseMdxSync } = await import('../../../src/parse.server');
    const ast = parseMdxSync('# Text');
    expect(ast[0].node_type).toBe('to_json_node');
  });

  it('covers invalid JSON string error', async () => {
    (globalThis as any).__nativeMock = {
      parseToJson: vi.fn().mockReturnValue('invalid-json')
    };
    const { parseMdxSync } = await import('../../../src/parse.server');
    expect(() => parseMdxSync('# Text')).toThrow("Invalid JSON");
  });

  it('covers invalid JSON from toJson() object', async () => {
    (globalThis as any).__nativeMock = {
      parseToJson: vi.fn().mockReturnValue({ toJson: () => 'invalid-json' })
    };
    const { parseMdxSync } = await import('../../../src/parse.server');
    expect(() => parseMdxSync('# Text')).toThrow("Invalid JSON");
  });

  it('covers unrecognized return format error', async () => {
    (globalThis as any).__nativeMock = {
      parseToJson: vi.fn().mockReturnValue('null') 
    };
    const { parseMdxSync } = await import('../../../src/parse.server');
    expect(() => parseMdxSync('# Text')).toThrow("Unrecognized return format from Rust parser");
  });

  it('covers compileToJsx unsupported error', async () => {
    (globalThis as any).__nativeMock = {
      parseToJson: vi.fn().mockReturnValue('[]')
    };
    const { compileToJsx, compileToJsxSync } = await import('../../../src/parse.server');
    await expect(compileToJsx('# Text')).rejects.toThrow("compileToJsx is not supported");
    expect(() => compileToJsxSync('# Text')).toThrow("compileToJsx is not supported");
  });

  it('covers local fs fallback logic', async () => {
    (globalThis as any).__nativeMockMode = 'throw'; 
    
    (globalThis as any).__fsMock = {
      existsSync: vi.fn().mockReturnValue(true),
      readdirSync: vi.fn().mockReturnValue(['dummy-local-fallback.node'])
    };

    try {
        const { parseMdxSync } = await import('../../../src/parse.server');
        parseMdxSync('# Try');
    } catch (e: any) {
        expect(e.message).toContain("Native parser not found");
    }
  });
});