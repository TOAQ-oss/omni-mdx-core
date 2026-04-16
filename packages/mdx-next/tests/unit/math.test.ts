import { describe, it, expect } from 'vitest';
import { parseMdxSync } from '../../src/server';

describe('Math Parsing (AST Generation)', () => {
  it('parses inline math ($...$) into the correct AST node', () => {
    const result = parseMdxSync('La formule est $E=mc^2$.');
    const strResult = JSON.stringify(result);

    expect(strResult).toMatch(/inlineMath|math/i);
    expect(strResult).toContain('E=mc^2');
  });

  it('parses block math ($$...$$) into the correct AST node', () => {
    const result = parseMdxSync('$$\n\\frac{a}{b}\n$$');
    const strResult = JSON.stringify(result);
    
    expect(strResult).toMatch(/math/i);
    expect(strResult).toContain('\\frac{a}{b}');
  });

  it('handles empty math blocks without crashing', () => {
    expect(() => parseMdxSync('$$$$')).not.toThrow();
    expect(() => parseMdxSync('$$')).not.toThrow();
  });
});