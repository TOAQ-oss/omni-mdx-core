import { describe, it, expect } from 'vitest';

import { parseMdx } from '../../src/client';

describe('WASM module – integration', () => {
  it('produces consistent output for the same input', () => {

    const a = parseMdx('# Hello');
    const b = parseMdx('# Hello');
    expect(a).toEqual(b);
  })

  it('handles unicode input', () => {
    expect(() => parseMdx('こんにちは 🌸 مرحبا')).not.toThrow();
  })

  it('handles large input without crashing', () => {
    const big = '# heading\n\nparagraph.\n\n'.repeat(10_000);
    expect(() => parseMdx(big)).not.toThrow();
  })
})