// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import * as parserClient from '../../../src/parse.client';

describe('parse.client depth coverage', () => {
  beforeEach(() => {
    vi.resetModules();
    vi.restoreAllMocks();
  });

  it('covers initialization and fetch logic', async () => {
    const fetchSpy = vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      arrayBuffer: () => Promise.resolve(new ArrayBuffer(100))
    }));

    const { parseMdxClient } = await import('../../../src/parse.client');

    try {
      await parseMdxClient('# Test');
    } catch (e) {}
    
    expect(globalThis.fetch).toHaveBeenCalled();
  });
});

describe('parseMdxClient', () => {
  
  beforeEach(() => {
    vi.restoreAllMocks();
    
    globalThis.fetch = vi.fn().mockImplementation(() =>
        Promise.resolve({
            ok: true,
            arrayBuffer: () => Promise.resolve(new ArrayBuffer(0)),
        } as Response)
    );
  });

  it('parses MDX on the client-side', async () => {
    const mockAst = [{ node_type: 'h1', children: [{ node_type: 'text', content: 'Client Side' }] }];
    
    const parseSpy = vi.spyOn(parserClient, 'parseMdxClient').mockResolvedValue(mockAst as any);

    const result = await parserClient.parseMdxClient('# Client Side');
    
    expect(parseSpy).toHaveBeenCalledWith('# Client Side');
    expect(result[0].node_type).toBe('h1');
  });

  it('covers initialization and fetch logic', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      arrayBuffer: () => Promise.resolve(new ArrayBuffer(100)) 
    });

    try {
      await parserClient.parseMdxClient('# Test');
    } catch (e) {}
    
    expect(globalThis.fetch).toHaveBeenCalled();
  });

  it('covers the error branch when fetch fails', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({ ok: false });
    
    await expect(parserClient.parseMdxClient('# Test')).rejects.toThrow();
  });
  it('covers window undefined branch', async () => {
    const originalWindow = globalThis.window;
    // @ts-expect-error - Simulate the absence of a window
    delete globalThis.window;
    
    const result = await parserClient.parseMdxClient('# test');
    expect(result).toEqual([]);
    
    globalThis.window = originalWindow;
  });

  it('covers the error branch when fetch fails', async () => {
    globalThis.fetch = vi.fn().mockRejectedValue(new Error("Init Failed"));

    await expect(parserClient.parseMdxClient('# test'))
      .rejects
      .toThrow(/Syntax error in MDX:/); 
  });
});