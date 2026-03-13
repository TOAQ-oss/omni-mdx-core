import { defineConfig } from "tsup";

export default defineConfig([
  {
    entry: {
      server: "src/server.ts",
      index:  "src/index.ts",
    },
    format: ["esm"],
    dts: false,
    splitting: false,
    sourcemap: true,
    clean: true,
    platform: "node",
    target: ["node18"],
    external: [
      "react",
      "react-dom",
      "react/jsx-runtime",
      "next",
      /\.node$/,
      /\.wasm$/,
    ],
    esbuildOptions(options) {
      options.jsx = "automatic";
    },
  },
  {
    entry: {
      client: "src/client.ts",
    },
    format: ["esm", "cjs"],
    dts: false,
    splitting: false,
    sourcemap: true,
    platform: "browser",
    target: ["es2020"],
    external: [
      "react",
      "react-dom",
      "react/jsx-runtime",
      "katex",
      "next",
    ],
    esbuildOptions(options) {
      options.jsx = "automatic";
      options.loader = { ...options.loader, ".wasm": "empty" };
    },
  },
]);