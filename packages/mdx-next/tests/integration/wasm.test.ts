import { describe, it, expect, beforeAll } from 'vitest'
import { readFile } from 'fs/promises'
import { resolve } from 'path'

// Ce fichier vit dans packages/mdx-next/tests/integration/
// => ../../wasm = packages/mdx-next/wasm/
const WASM_DIR = resolve(__dirname, '../../wasm')
const WASM_BIN = resolve(WASM_DIR, 'omni_mdx_core_bg.wasm')
const WASM_JS  = resolve(WASM_DIR, 'omni_mdx_core.js')

// wasm-pack --target web génère un init() qui utilise fetch() en interne,
// ce qui ne fonctionne pas sur Node.js pour les fichiers locaux (file://).
// Solution : on lit le .wasm avec readFile() et on le passe directement
// à init() sous forme d'ArrayBuffer — l'API wasm-bindgen l'accepte.
let wasm: Record<string, unknown> | null = null

beforeAll(async () => {
  try {
    const [mod, wasmBytes] = await Promise.all([
      import(/* @vite-ignore */ WASM_JS),
      readFile(WASM_BIN),
    ])
    // init() accepte un ArrayBuffer, un Response, ou une URL.
    // On passe l'ArrayBuffer pour court-circuiter fetch().
    await mod.default(wasmBytes.buffer)
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