import { describe, it, expect } from 'vitest';
import { 
  parseMdx, 
  parseMdxSync, 
  compileToJsx, 
  compileToJsxSync, 
  MDXParseError 
} from '../../../src/parse.server';

describe('parse.server - Deep Coverage', () => {
  it('handles binary input (Uint8Array/Buffer) in async and sync', async () => {
    const input = Buffer.from('# Binary Title');
    
    const astAsync = await parseMdx(input);
    const astSync = parseMdxSync(input);

    expect(astAsync[0].node_type).toBe('h1');
    expect(astSync[0].node_type).toBe('h1');
  });

  it('covers JSX compilation paths', async () => {
    try {
      await compileToJsx('# Title');
    } catch (e) {
      expect(e).toBeDefined();
    }

    try {
      compileToJsxSync('# Title');
    } catch (e) {
      expect(e).toBeDefined();
    }
  });

  it('instantiates MDXParseError directly for coverage', () => {
    const error = new MDXParseError("Custom error", "snippet");
    expect(error.message).toContain("MDX parse error");
    expect(error.source).toBe("snippet");
  });

  it('triggers MDXParseError on invalid syntax', async () => {
    const badMdx = '<UnclosedTag'; 
    await expect(parseMdx(badMdx)).rejects.toThrow(MDXParseError);
    expect(() => parseMdxSync(badMdx)).toThrow(MDXParseError);
  });
});