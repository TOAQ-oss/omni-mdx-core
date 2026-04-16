import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    coverage: {
      provider: 'v8',
      reporter: ['lcov', 'text'],
      reportsDirectory: './coverage',
      include: ['src/**'],
      // 2. On exclut tout ce qui pourrait polluer, avec des patterns universels
      exclude: [
        '**/node_modules/**',
        '**/dist/**',
        '**/wasm/**',           // Cible le dossier wasm n'importe où
        '**/*.d.ts',            // Exclut les fichiers de types
        'src/types/**',         // Souvent inutile de couvrir des interfaces
        '**/omni_mdx_core.js',  // Cible directe le fichier rebelle
      ],
      // 3. Force le rafraîchissement
      clean: true,
    },
  },
})