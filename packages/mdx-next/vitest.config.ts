import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    coverage: {
      provider: 'v8',
      reporter: ['lcov', 'text'],
      reportsDirectory: './coverage',
      include: ['src/**'],
      exclude: [
        '**/node_modules/**',
        '**/dist/**',
        '**/wasm/**',
        '**/*.d.ts',
        'src/types/**',
        '**/omni_mdx_core.js',
      ],
      clean: true,
    },
  },
})