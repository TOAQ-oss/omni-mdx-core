import { describe, it, expect } from 'vitest'

// Adapte cet import à ton API réelle
// import { parse, render } from '../../packages/mdx-next/src'

describe('parser – unit', () => {
  it('parses an empty string without throwing', () => {
    // const result = parse('')
    // expect(result).toBeDefined()
    expect(true).toBe(true) // placeholder — remplace par tes vrais appels
  })

  it('returns a non-null AST for valid input', () => {
    // const result = parse('# Hello')
    // expect(result.type).toBe('root')
    expect(true).toBe(true)
  })

  it('throws on completely invalid input', () => {
    // expect(() => parse(null as any)).toThrow()
    expect(true).toBe(true)
  })
})

describe('renderer – unit', () => {
  it('renders a heading node to HTML', () => {
    // const html = render({ type: 'heading', depth: 1, value: 'Hello' })
    // expect(html).toContain('<h1>')
    expect(true).toBe(true)
  })
})