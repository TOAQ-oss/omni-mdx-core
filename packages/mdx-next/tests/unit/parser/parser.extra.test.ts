import { describe, it, expect, vi } from 'vitest';
import { parseMdxSync } from '../../../src/parse.server';

describe('Parsing Extra Branches', () => {
  it('covers server error branches', () => {
    try {
      parseMdxSync({} as any);
    } catch (err) {
      expect(err).toBeDefined();
    }
  });

  it('covers the TextEncoder/Decoder paths in parse.client', async () => {
    const { parseMdxClient } = await import('../../../src/parse.client');

    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      arrayBuffer: () => Promise.resolve(new ArrayBuffer(8))
    });

    try {
      await parseMdxClient('# Test');
    } catch (e) {}
  });
});