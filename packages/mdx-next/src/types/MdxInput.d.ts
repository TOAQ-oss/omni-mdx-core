interface SerializedBuffer {
  type: "Buffer";
  data: number[];
}

type MdxInput = string | Buffer | Uint8Array | SerializedBuffer;

export {
    SerializedBuffer,
    MdxInput
}