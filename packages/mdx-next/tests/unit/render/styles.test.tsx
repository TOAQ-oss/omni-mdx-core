import { describe, it, expect } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { parseMdxSync } from '../../../src/server';
import { MDXServerRenderer } from '../../../src/MDXServerRenderer';

describe('Basic Styles Coverage', () => {
  it('renders every single basic style component to ensure 100% coverage', () => {
    const fullMdx = [
      '# H1',
      '## H2',
      '### H3',
      '#### H4',
      'Paragraphe simple avec du `code` et un [lien](https://test.com)',
      '',
      '- Liste ul',
      '1. Liste ol',
      '',
      '> Blockquote',
      '',
      '---', // HR
      '',
      '![Alt text](https://image.com/img.png)',
      '',
      '| Header |',
      '| ------ |',
      '| Cell   |'
    ].join('\n');

    const ast = parseMdxSync(fullMdx);
    const html = renderToStaticMarkup(<MDXServerRenderer ast={ast} />);

    expect(html).toContain('text-3xl'); // h1
    expect(html).toContain('text-2xl'); // h2
    expect(html).toContain('text-xl');  // h3
    expect(html).toContain('text-lg');  // h4
    expect(html).toContain('list-disc'); // ul
    expect(html).toContain('list-decimal'); // ol
    expect(html).toContain('inline-code'); // code
    expect(html).toContain('border-l-4'); // blockquote
    expect(html).toContain('border-white/5'); // hr
    expect(html).toContain('rounded-xl'); // img
    expect(html).toContain('text-blue-400'); // a
    expect(html).toContain('overflow-x-auto'); // table container
  });
});