import { describe, it, expect } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { parseMdxSync } from '../../src/server';
import { MDXServerRenderer } from '../../src/MDXServerRenderer';

describe('Full MDX Pipeline Integration', () => {
  
  it('converts raw MDX string to final styled HTML', () => {
    const rawMdx = `
# Welcome
This is **bold** and this is $E=mc^2$.

<CustomCard title="Test">
  Nested content
</CustomCard>
    `.trim();

    const ast = parseMdxSync(rawMdx);

    const CustomCard = ({ title, children }: any) => (
      <div className="card">
        <h2>{title}</h2>
        {children}
      </div>
    );

    const html = renderToStaticMarkup(
      <MDXServerRenderer 
        ast={ast} 
        components={{ CustomCard }} 
      />
    );

    expect(html).toContain('class="omni-mdx-root"');
    expect(html).toMatch(/<h1[^>]*>Welcome<\/h1>/);
    expect(html).toMatch(/<strong[^>]*>bold<\/strong>/);
    expect(html).toContain('katex');
    expect(html).toContain('class="card"');
    expect(html).toMatch(/<h2[^>]*>Test<\/h2>/);
    expect(html).toContain('Nested content');
  });
});