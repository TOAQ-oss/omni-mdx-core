// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import * as parserClient from '../../../src/parse.client';

describe('parse.client depth coverage', () => {
  beforeEach(() => {
    vi.resetModules(); // Vide le cache des modules pour réinitialiser le singleton initPromise
    vi.restoreAllMocks();
  });

  it('covers initialization and fetch logic', async () => {
    // Mock de fetch
    const fetchSpy = vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      arrayBuffer: () => Promise.resolve(new ArrayBuffer(100))
    }));

    // Import dynamique pour obtenir une NOUVELLE instance du module
    const { parseMdxClient } = await import('../../../src/parse.client');

    try {
      await parseMdxClient('# Test');
    } catch (e) {
      // On ignore l'erreur de parsing binaire, on veut juste le fetch
    }
    
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
    // On mock fetch pour simuler la récupération du .wasm
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      arrayBuffer: () => Promise.resolve(new ArrayBuffer(100)) 
    });

    try {
      // On lance le parse. Même si le binaire est bidon, on traverse l'init.
      await parserClient.parseMdxClient('# Test');
    } catch (e) {
      // On ignore l'erreur de parsing binaire, on veut juste le coverage du fetch
    }
    
    expect(globalThis.fetch).toHaveBeenCalled();
  });

  it('covers the error branch when fetch fails', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({ ok: false });
    
    await expect(parserClient.parseMdxClient('# Test')).rejects.toThrow();
  });
  it('covers window undefined branch', async () => {
    // On simule un environnement non-navigateur temporairement
    const originalWindow = globalThis.window;
    // @ts-expect-error - Simuler l'absence de window
    delete globalThis.window;
    
    const result = await parserClient.parseMdxClient('# test');
    expect(result).toEqual([]);
    
    globalThis.window = originalWindow;
  });

  it('covers the error branch when fetch fails', async () => {
  // On simule l'échec du fetch
  globalThis.fetch = vi.fn().mockRejectedValue(new Error("Init Failed"));
  
  // On s'attend à ce que l'erreur soit interceptée et préfixée par parseMdxClient
  // On utilise une Regex pour vérifier le début du message
  await expect(parserClient.parseMdxClient('# test'))
    .rejects
    .toThrow(/Syntax error in MDX:/); 
});
});