import type { AstNode, AttrValueKind } from "../types/MdxAST";

// --- Binary Opcodes ---

/** Opcode indicating the node is a plain text node. */
const NODE_TEXT = 0x01;
/** Opcode indicating the node is an element (HTML tag or custom component). */
const NODE_ELEMENT = 0x02;

/** Opcode indicating an attribute with a standard string value. */
const ATTR_TEXT = 0x10;
/** Opcode indicating an attribute containing a JS/React expression (e.g., `prop={value}`). */
const ATTR_EXPRESSION = 0x11;
/** Opcode indicating a boolean attribute without a value (e.g., `disabled`). */
const ATTR_BOOLEAN = 0x12;
/** Opcode indicating an attribute that contains a nested AST inside its value. */
const ATTR_AST = 0x13;

/**
 * A highly optimized binary decoder for Omni-MDX.
 * * It deserializes the custom binary payload produced by the Rust core parser 
 * and reconstructs the JavaScript Abstract Syntax Tree (AST). This avoids the 
 * massive overhead of JSON stringification and parsing, especially for large documents.
 */
export class MdxBinaryDecoder {
  private view: DataView;
  private buffer: Uint8Array;
  private offset: number = 0;
  private decoder = new TextDecoder("utf-8");

  /** * String interning cache.
   * Stores frequently used strings (like tag names or attribute keys) to avoid 
   * allocating duplicate strings in memory, significantly reducing Garbage Collection overhead.
   */
  private stringCache = new Map<string, string>();

  /**
   * Initializes a new binary decoder.
   * @param buffer - The raw binary data array returned by the Rust/WASM parser.
   */
  constructor(buffer: Uint8Array) {
    this.buffer = buffer;
    this.view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  }


  /**
   * Starts the decoding process.
   * @returns An array of parsed `AstNode` objects representing the root of the document.
   */
  public decode(): AstNode[] {
    const rootCount = this.readU32();
    const nodes: AstNode[] = [];
    for (let i = 0; i < rootCount; i++) {
      nodes.push(this.decodeNode());
    }
    return nodes;
  }

  /**
   * Recursively decodes a single AST node and its children/attributes.
   * @returns The constructed `AstNode`.
   * @throws {Error} If an unrecognized binary opcode is encountered.
   */
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

  /**
   * Reads an unsigned 8-bit integer (1 byte) and advances the offset.
   * @returns The numeric value.
   */
  private readU8(): number {
    const val = this.view.getUint8(this.offset);
    this.offset += 1;
    return val;
  }

  /**
   * Reads an unsigned 16-bit integer (2 bytes, little-endian) and advances the offset.
   * @returns The numeric value.
   */
  private readU16(): number {
    const val = this.view.getUint16(this.offset, true);
    this.offset += 2;
    return val;
  }

  /**
   * Reads an unsigned 32-bit integer (4 bytes, little-endian) and advances the offset.
   * @returns The numeric value.
   */
  private readU32(): number {
    const val = this.view.getUint32(this.offset, true);
    this.offset += 4;
    return val;
  }

  /**
   * Reads a UTF-8 string prefixed by a 16-bit length indicator.
   * Uses a string cache (interning) to prevent allocating duplicate string instances.
   * @returns The decoded string.
   */
  private readStringU16(): string {
    const len = this.readU16();
    if (this.offset + len > this.buffer.byteLength) {
      throw new Error(`Binary corruption: string (u16) out of bounds at offset ${this.offset}`);
    }
    const str = this.decoder.decode(this.buffer.subarray(this.offset, this.offset + len));
    this.offset += len;

    let cached = this.stringCache.get(str);
    if (cached) return cached;
    this.stringCache.set(str, str);
    return str;
  }

  /**
   * Reads a UTF-8 string prefixed by a 32-bit length indicator.
   * Ideal for potentially massive strings like long text nodes or code blocks.
   * @returns The decoded string.
   */
  private readStringU32(): string {
    const len = this.readU32();
    if (this.offset + len > this.buffer.byteLength) {
      throw new Error(`Binary corruption: string (u32) out of bounds at offset ${this.offset}`);
    }
    const str = this.decoder.decode(this.buffer.subarray(this.offset, this.offset + len));
    this.offset += len;
    return str;
  }
}