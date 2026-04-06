interface SerializedBuffer {
  type: "Buffer";
  data: number[];
}

type MdxInput = string | Buffer | Uint8Array | SerializedBuffer;

import type { Plugin } from 'unified';

export type OmniMdxOptions = {
  rehypePlugins?: Plugin[];
};

export {
    SerializedBuffer,
    MdxInput
}