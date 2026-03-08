import { defineConfig } from 'tsup';
import cssModulesPlugin from 'esbuild-css-modules-plugin';

export default defineConfig({
  entry: ['src/index.tsx'],
  format: ['cjs', 'esm'],
  dts: true,
  external: ['react'],
  injectStyle: true,
  esbuildPlugins: [cssModulesPlugin()]
});