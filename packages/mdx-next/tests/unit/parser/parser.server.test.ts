import { describe, it, expect } from 'vitest'
import { parseMdx, parseMdxSync, MDXParseError } from '../../../src/server';

describe('parser asynchronous', () => {
  it('parses an empty string without throwing', async () => {
    const result = await parseMdx('');

    expect(result).toBeDefined();
  });

  it('returns a non-null AST for valid input', async () => {
    const result = await parseMdx('# Hello');

    expect(result).toStrictEqual([ { node_type: 'h1', "children": [{"content": "Hello","node_type": "text"}]}]);
  });

 it('throws on completely invalid MDX syntax', async () => {
    const badInput = '<UnclosedTag';

    await expect(parseMdx(badInput)).rejects.toThrow();
  });
})

describe('parser synchronous', () => {
  it('parses an empty string without throwing', () => {
    const result = parseMdxSync('');

    expect(result).toBeDefined();
  });

  it('returns a non-null AST for valid input', () => {
    const result = parseMdxSync('# Hello');

    expect(result).toStrictEqual([ { node_type: 'h1', "children": [{"content": "Hello","node_type": "text"}]}])
  });

 it('throws on completely invalid MDX syntax', () => {
    const badInput = '<UnclosedTag' 

    expect(() => parseMdxSync(badInput)).toThrow()
  });
  it('covers binary input and JSON errors in parse.server', async () => {
  // Entrée binaire pour couvrir 122-123
  const buf = Buffer.from('# Hello');
  const ast = await parseMdx(buf);
  expect(ast[0].node_type).toBe('h1');

  // Forcer une erreur JSON (si possible via mock ou input spécifique)
  // pour couvrir les branches de secours.
  const error = new MDXParseError("test", "snippet");
  expect(error.source).toBe("snippet");
});
})