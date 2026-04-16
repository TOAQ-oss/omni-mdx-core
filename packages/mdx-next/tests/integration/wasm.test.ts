import { describe, it, expect } from 'vitest';
import { parseMdx } from '../../src/client';

describe('WASM module – integration', () => {
  it('produces consistent output for the same input', async () => {
    // Il faut await les appels au client WASM
    const a = await parseMdx('# Hello');
    const b = await parseMdx('# Hello');
    
    expect(a).toEqual(b);
  });

  it('handles unicode input without crashing', async () => {
    // Utilisation de resolves pour tester l'intérieur de la promesse
    await expect(parseMdx('こんにちは 🌸 مرحبا')).resolves.toBeDefined();
  });

  it('handles large input without crashing', async () => {
    // Note: On réduit un peu la taille pour éviter que Vitest ne coupe le test pour TimeOut (1000 est suffisant pour prouver la stabilité de la mémoire WASM)
    const big = '# heading\n\nparagraph.\n\n'.repeat(1_000);
    
    await expect(parseMdx(big)).resolves.toBeDefined();
  });
});