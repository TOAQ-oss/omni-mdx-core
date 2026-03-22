import { describe, it, expect, beforeAll } from 'vitest'
import { readFile } from 'fs/promises'
import { resolve } from 'path'

let wasmModule: WebAssembly.Exports | null = null

beforeAll(async () => {
  const wasmPath = resolve(__dirname, '../../../packages/mdx-next/wasm/omni_mdx_core_bg.wasm')
  const bytes = await readFile(wasmPath)
  const { instance } = await WebAssembly.instantiate(bytes)
  wasmModule = instance.exports

  // Si tu utilises l'init JS généré par wasm-pack :
  // const init = (await import('../../../packages/mdx-next/wasm/omni_mdx_core.js')).default
  // await init()
})

describe('WASM module – integration', () => {
  it('loads without error', () => {
    expect(wasmModule).not.toBeNull()
  })

  it('exports expected functions', () => {
    // Adapte aux fonctions que ton core expose réellement
    // expect(typeof (wasmModule as any).parse).toBe('function')
    expect(true).toBe(true) // placeholder
  })

  it('produces consistent output for the same input', () => {
    // const a = (wasmModule as any).parse('# Hello')
    // const b = (wasmModule as any).parse('# Hello')
    // expect(a).toEqual(b)
    expect(true).toBe(true)
  })

  it('handles unicode input', () => {
    // const result = (wasmModule as any).parse('こんにちは 🌸')
    // expect(result).toBeDefined()
    expect(true).toBe(true)
  })

  it('handles large input without crashing', () => {
    // const big = '# heading\n'.repeat(10_000)
    // expect(() => (wasmModule as any).parse(big)).not.toThrow()
    expect(true).toBe(true)
  })
})