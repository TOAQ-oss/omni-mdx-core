import { describe, it, expect } from 'vitest';
import { MdxBinaryDecoder } from '../../src/utils/binaryDecoder';

describe('MdxBinaryDecoder - Advanced Nodes', () => {
  it('decodes elements with multiple attribute types and nesting', () => {
    const buffer = new Uint8Array([
      1, 0, 0, 0,             // u32 rootCount = 1
      0x02,                   // u8 type = NODE_ELEMENT
      3, 0,                   // u16 tagLen = 3
      100, 105, 118,          // "div"
      0,                      // u8 self_closing = false
      2, 0,                   // u16 attrCount = 2
      
      // Attr 1: Boolean
      4, 0, 100, 97, 116, 97, // u16 keyLen=4, "data"
      0x12,                   // u8 kind = ATTR_BOOLEAN
      
      // Attr 2: Expression
      2, 0, 105, 100,         // u16 keyLen=2, "id"
      0x11,                   // u8 kind = ATTR_EXPRESSION
      1, 0, 0, 0, 49,         // u32 valLen=1, "1"

      1, 0, 0, 0,             // u32 childCount = 1
      0x01,                   // u8 child type = NODE_TEXT
      2, 0, 0, 0, 111, 107    // u32 stringLen = 2, "ok"
    ]);

    const decoder = new MdxBinaryDecoder(buffer);
    const result = decoder.decode();

    expect(result[0].node_type).toBe('div');
    expect(result[0].attributes).toMatchObject({
      data: { kind: 'boolean' },
      id: { kind: 'expression', value: '1' }
    });
  });
  it('throws error on truncated buffer during string read', () => {
    const truncated = new Uint8Array([1, 0, 0, 0, 0x01, 100, 0, 0, 0, 65, 66]);
    const decoder = new MdxBinaryDecoder(truncated);
    expect(() => decoder.decode()).toThrow();
    });
});