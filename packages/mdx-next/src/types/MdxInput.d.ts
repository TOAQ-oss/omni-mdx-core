interface SerializedBuffer {
  type: "Buffer";
  data: number[];
}

type MdxInput = string | Buffer | Uint8Array | SerializedBuffer;

import type { PluggableList } from 'unified';

export type OmniMdxOptions = {
  rehypePlugins?: PluggableList;
};

export {
    SerializedBuffer,
    MdxInput
}