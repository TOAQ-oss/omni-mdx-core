import { describe, it, expect } from 'vitest';
import { parseMdx, parseMdxSync } from '../../src/server';

describe('Native Bindings Integration (Sync vs Async Parity)', () => {
  
  it('produces identical ASTs for basic Markdown', async () => {
    const mdxContent = '# Titre Principal\nVoici un paragraphe simple avec du **gras** et de l\'*italique*.';
    expect(parseMdxSync(mdxContent)).toEqual(await parseMdx(mdxContent));
  });

  it('produces identical ASTs for complex MDX (Components, Props, Math)', async () => {
    const complexMdx = '<HeroBanner theme="dark" showTitle>\n  # Welcome\n  $E=mc^2$\n</HeroBanner>';
    expect(parseMdxSync(complexMdx)).toEqual(await parseMdx(complexMdx));
  });

  it('handles empty strings identically', async () => {
    expect(parseMdxSync('')).toEqual(await parseMdx(''));
  });

  it('handles syntax errors with parity (throws vs rejects with exact same message)', async () => {
    const badMdx = '<UnclosedComponent prop="test"> Voici un texte';
    
    let syncErrorMsg = '';
    let asyncErrorMsg = '';

    try { parseMdxSync(badMdx); } catch (e: any) { syncErrorMsg = e.message; }
    try { await parseMdx(badMdx); } catch (e: any) { asyncErrorMsg = e.message; }

    expect(syncErrorMsg).toBeTruthy();
    expect(syncErrorMsg).toEqual(asyncErrorMsg);
  });
});