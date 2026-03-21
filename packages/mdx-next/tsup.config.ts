import { defineConfig } from "tsup";
import { copyFileSync, mkdirSync } from "fs";
import { resolve } from "path";

export default defineConfig([
  {
    entry: {
      server: "src/server.ts",
      index:  "src/index.ts",
    },
    format: ["esm"],
    dts: true,
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
    dts: true,
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
    async onSuccess() {
      try {
        mkdirSync("dist", { recursive: true });
        copyFileSync(
          resolve("wasm/omni_mdx_core_bg.wasm"),
          resolve("dist/omni_mdx_core_bg.wasm")
        );
        console.log("✓ Copied omni_mdx_core_bg.wasm → dist/");
      } catch (e) {
        console.warn("⚠ Could not copy .wasm file:", e);
      }
    },
  },
]);