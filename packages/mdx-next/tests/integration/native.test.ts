import { describe, it, expect } from 'vitest';
import { parseMdx, parseMdxSync } from '../../src/server';

describe('Native Bindings Integration (Sync vs Async Parity)', () => {
  
  it('produces identical ASTs for basic Markdown', async () => {
    const mdxContent = `
# Titre Principal
Voici un paragraphe simple avec du **gras** et de l'*italique*.
    `.trim();

    const syncAst = parseMdxSync(mdxContent);
    const asyncAst = await parseMdx(mdxContent);

    expect(syncAst).toEqual(asyncAst);

    expect(syncAst.length).toBeGreaterThan(0);
    expect(syncAst[0].node_type).toBe('h1');
  });

  it('produces identical ASTs for complex MDX (Components, Props, Math)', async () => {
    const complexMdx = `
<HeroBanner theme="dark" showTitle>
  # Welcome
  La formule de l'énergie est $E=mc^2$.
</HeroBanner>

$$
\\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}
$$
    `.trim();

    const syncAst = parseMdxSync(complexMdx);
    const asyncAst = await parseMdx(complexMdx);

    expect(syncAst).toEqual(asyncAst);
    expect(syncAst[0].node_type).toBe('HeroBanner');
  });

  it('handles empty strings identically', async () => {
    const syncAst = parseMdxSync('');
    const asyncAst = await parseMdx('');

    expect(syncAst).toEqual(asyncAst);
    expect(syncAst).toEqual([]);
  });

  it('handles syntax errors with parity (throws vs rejects)', async () => {
    const badMdx = '<UnclosedComponent prop="test"> Voici un texte';

    expect(() => parseMdxSync(badMdx)).toThrow();

    await expect(parseMdx(badMdx)).rejects.toThrow();
  });

  it('throws the exact same error message for both methods', async () => {
    const badMdx = '<div>Balise mal fermée</span>';
    
    let syncErrorMsg = '';
    let asyncErrorMsg = '';

    try {
      parseMdxSync(badMdx);
    } catch (e: any) {
      syncErrorMsg = e.message;
    }

    try {
      await parseMdx(badMdx);
    } catch (e: any) {
      asyncErrorMsg = e.message;
    }

    expect(syncErrorMsg).toBeTruthy();
    expect(syncErrorMsg).toEqual(asyncErrorMsg);
  });

});