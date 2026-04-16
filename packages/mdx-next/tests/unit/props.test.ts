import { describe, it, expect } from 'vitest';
import { parseProps } from '../../src/utils/parserHelpers'; 

describe('Props Parsing – parseProps', () => {

  it('returns non-string values immediately (passthrough)', () => {
    expect(parseProps(42 as any)).toBe(42);
    expect(parseProps(true as any)).toBe(true);
    expect(parseProps({ a: 1 } as any)).toEqual({ a: 1 });
  });

  it('parses primitive strings (booleans and numbers)', () => {
    expect(parseProps('true')).toBe(true);
    expect(parseProps('false')).toBe(false);
    expect(parseProps('42')).toBe(42);
    expect(parseProps('3.14')).toBe(3.14);
  });

  it('parses standard strings and leaves them untouched', () => {
    expect(parseProps('Hello World')).toBe('Hello World');
    expect(parseProps('title-class')).toBe('title-class');
  });

  it('evaluates valid JSON5 arrays and objects', () => {
    expect(parseProps('{ theme: "dark", active: true }')).toEqual({ theme: 'dark', active: true });
    expect(parseProps('[1, 2, "three"]')).toEqual([1, 2, 'three']);
  });

  it('unwraps double JSX braces for objects and arrays', () => {
    expect(parseProps('{{ a: 1, b: 2 }}')).toEqual({ a: 1, b: 2 });
    expect(parseProps('{[ "apple", "banana" ]}')).toEqual(['apple', 'banana']);
  });

  it('falls back to returning the raw string if JSON5 parsing fails', () => {
    const malformedObject = '{ theme: "dark", brokenKey: }';
    expect(parseProps(malformedObject)).toBe(malformedObject);
    
    const malformedArray = '[1, 2, 3';
    expect(parseProps(malformedArray)).toBe(malformedArray);
  });

});