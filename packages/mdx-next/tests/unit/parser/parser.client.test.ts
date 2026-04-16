// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import * as wasm from '../../../wasm/omni_mdx_core.js';
import { MdxBinaryDecoder } from '../../../src/utils/binaryDecoder';
import { runUnifiedPipeline } from '../../../src/utils/unifiedBridge';

vi.mock('../../../wasm/omni_mdx_core.js', () => ({
  default: vi.fn().mockResolvedValue(true),
  parse_to_binary: vi.fn()
}));

vi.mock('../../../src/utils/binaryDecoder', () => ({
  MdxBinaryDecoder: vi.fn().mockImplementation(function() {
    return {
      decode: vi.fn()
    };
  })
}));

vi.mock('../../../src/utils/unifiedBridge', () => ({
  runUnifiedPipeline: vi.fn()
}));

describe('MDX Client Parser', () => {
 beforeEach(() => {
    vi.clearAllMocks();
    
    vi.mocked(wasm.parse_to_binary).mockReturnValue(new Uint8Array([0]));
    vi.mocked(MdxBinaryDecoder).mockImplementation(function() {
      return {
        decode: () => [{ node_type: 'h1' }]
      } as any;
    });

    vi.mocked(runUnifiedPipeline).mockResolvedValue([{ node_type: 'p', content: 'unified' }]);
  });

  it('covers window undefined branch', async () => {
    const originalWindow = globalThis.window;
    // @ts-expect-error - We simulate a windowless environment (e.g., pure Node)
    delete globalThis.window;
    
    const { parseMdxClient } = await import('../../../src/parse.client');
    const result = await parseMdxClient('# test');
    expect(result).toEqual([]);
    
    globalThis.window = originalWindow;
  });

  it('normalizes inputs and covers singleton WASM init', async () => {
    const { parseMdxClient } = await import('../../../src/parse.client');
    
    const ast1 = await parseMdxClient('# Test');
    expect(ast1[0].node_type).toBe('h1');
    
    await parseMdxClient(new Uint8Array([35, 32, 84]));
    await parseMdxClient({ type: 'Buffer', data: [35, 32, 84] } as any);
    await parseMdxClient(123 as any);
  });

  it('covers rehypePlugins branch', async () => {
    const { parseMdxClient } = await import('../../../src/parse.client');
    const dummyPlugin = () => (tree: any) => tree;
    
    const result = await parseMdxClient('# Test', { rehypePlugins: [dummyPlugin] });
    expect(result[0].node_type).toBe('p'); 
  });

  it('throws syntax error when parsing fails with an Error instance', async () => {
    vi.mocked(wasm.parse_to_binary).mockImplementation(() => {
      throw new Error("Core Failure");
    });

    const { parseMdxClient } = await import('../../../src/parse.client');
    await expect(parseMdxClient('# test')).rejects.toThrow(/Syntax error/);
  });

  it('throws syntax error when parsing fails with a non-Error object', async () => {
    vi.mocked(wasm.parse_to_binary).mockImplementation(() => {
      throw "String Error";
    });

    const { parseMdxClient } = await import('../../../src/parse.client');
    await expect(parseMdxClient('# test')).rejects.toThrow(/Syntax error in MDX: String Error/); 
  });
});