import type { AstNode, AttrValueKind } from "../types/MdxAST";

const NODE_TEXT = 0x01;
const NODE_ELEMENT = 0x02;

const ATTR_TEXT = 0x10;
const ATTR_EXPRESSION = 0x11;
const ATTR_BOOLEAN = 0x12;
const ATTR_AST = 0x13;

export class MdxBinaryDecoder {
  private view: DataView;
  private buffer: Uint8Array;
  private offset: number = 0;
  private decoder = new TextDecoder("utf-8");
  private stringCache = new Map<string, string>();

  constructor(buffer: Uint8Array) {
    this.buffer = buffer;
    this.view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  }

  public decode(): AstNode[] {
    const rootCount = this.readU32();
    const nodes: AstNode[] = [];
    for (let i = 0; i < rootCount; i++) {
      nodes.push(this.decodeNode());
    }
    return nodes;
  }

  private decodeNode(): AstNode {
    const type = this.readU8();

    if (type === NODE_TEXT) {
      return {
        node_type: "text",
        content: this.readStringU32(),
      };
    }

    if (type === NODE_ELEMENT) {
      const node_type = this.readStringU16();
      const self_closing = this.readU8() === 1;
      const attrCount = this.readU16();
      
      let attributes: Record<string, AttrValueKind> | undefined = undefined;
      
      if (attrCount > 0) {
        attributes = {};
        for (let i = 0; i < attrCount; i++) {
          const key = this.readStringU16();
          const attrKind = this.readU8();
          
          if (attrKind === ATTR_TEXT) {
            attributes[key] = { kind: "text", value: this.readStringU32() };
          } else if (attrKind === ATTR_EXPRESSION) {
            attributes[key] = { kind: "expression", value: this.readStringU32() };
          } else if (attrKind === ATTR_BOOLEAN) {
            attributes[key] = { kind: "boolean" };
          } else if (attrKind === ATTR_AST) {
            const subNodeCount = this.readU32();
            const subNodes: AstNode[] = [];
            for (let j = 0; j < subNodeCount; j++) {
              subNodes.push(this.decodeNode());
            }
            attributes[key] = { kind: "ast", value: subNodes };
          }
        }
      }

      const childCount = this.readU32();
      const children: AstNode[] = [];
      if (childCount > 0) {
        for (let i = 0; i < childCount; i++) {
          children.push(this.decodeNode());
        }
      }

      return {
        node_type,
        self_closing,
        attributes,
        children,
      };
    }

    throw new Error(`[@toaq-oss/omni-mdx] Unknown binary opcode: ${type} at offset ${this.offset}`);
  }

  private readU8(): number {
    const val = this.view.getUint8(this.offset);
    this.offset += 1;
    return val;
  }

  private readU16(): number {
    const val = this.view.getUint16(this.offset, true);
    this.offset += 2;
    return val;
  }

  private readU32(): number {
    const val = this.view.getUint32(this.offset, true);
    this.offset += 4;
    return val;
  }

  private readStringU16(): string {
    const len = this.readU16();
    const str = this.decoder.decode(this.buffer.subarray(this.offset, this.offset + len));
    this.offset += len;

    let cached = this.stringCache.get(str);
    if (cached) return cached;
    this.stringCache.set(str, str);
    return str;
  }

  private readStringU32(): string {
    const len = this.readU32();
    const str = this.decoder.decode(this.buffer.subarray(this.offset, this.offset + len));
    this.offset += len;
    return str;
  }
}