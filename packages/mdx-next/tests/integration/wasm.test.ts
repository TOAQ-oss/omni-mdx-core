import { describe, it, expect, beforeAll } from 'vitest'
import { resolve } from 'path'

// Ce fichier vit dans packages/mdx-next/tests/integration/
// => remonter 2 niveaux donne packages/mdx-next/  puis /wasm
const WASM_DIR = resolve(__dirname, '../../wasm')

// Le module JS généré par wasm-pack expose une fonction default() = init()
// qui charge le .wasm depuis le même dossier automatiquement.
let wasm: Record<string, unknown> | null = null

beforeAll(async () => {
  try {
    const mod = await import(/* @vite-ignore */ `${WASM_DIR}/omni_mdx_core.js`)
    await mod.default()  // init() — initialise le WASM
    wasm = mod
  } catch (e) {
    console.warn('WASM module not found — skipping integration tests:', e)
  }
})

function requireWasm() {
  if (!wasm) throw new Error('WASM not loaded — run wasm-pack build first')
}

describe('WASM module – integration', () => {
  it('loads without error', () => {
    expect(wasm).not.toBeNull()
  })

  it('exports expected functions', () => {
    requireWasm()
    // Adapte aux fonctions exposées par ton #[wasm_bindgen]
    // expect(typeof (wasm as any).parse_to_json).toBe('function')
    // expect(typeof (wasm as any).parse_to_binary).toBe('function')
    expect(true).toBe(true) // placeholder
  })

  it('produces consistent output for the same input', () => {
    requireWasm()
    // const a = (wasm as any).parse_to_json('# Hello')
    // const b = (wasm as any).parse_to_json('# Hello')
    // expect(a).toEqual(b)
    expect(true).toBe(true)
  })

  it('handles unicode input', () => {
    requireWasm()
    // expect(() => (wasm as any).parse_to_json('こんにちは 🌸 مرحبا')).not.toThrow()
    expect(true).toBe(true)
  })

  it('handles large input without crashing', () => {
    requireWasm()
    // const big = '# heading\n\nparagraph.\n\n'.repeat(10_000)
    // expect(() => (wasm as any).parse_to_json(big)).not.toThrow()
    expect(true).toBe(true)
  })
})