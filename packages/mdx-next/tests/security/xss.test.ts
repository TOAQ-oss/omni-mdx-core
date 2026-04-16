import { describe, it, expect, vi } from 'vitest';
import { parseMdxSync } from '../../src/server';
import { renderToStaticMarkup } from 'react-dom/server';
import React from 'react';
import { MDXServerRenderer } from '../../src/MDXServerRenderer';

describe('Security - XSS Prevention', () => {
  
  it('should not execute script tags injected in MDX', () => {
    const maliciousMdx = '# Title\n<script>alert("xss")</script>\nContent';
    const ast = parseMdxSync(maliciousMdx);
    const html = renderToStaticMarkup(React.createElement(MDXServerRenderer, { ast }));

    expect(html).not.toContain('<script>alert("xss")</script>');
  });

  it('should neutralize javascript: URLs in links', () => {
    const maliciousLink = '[Click me](javascript:alert("xss"))';
    const ast = parseMdxSync(maliciousLink);
    const html = renderToStaticMarkup(React.createElement(MDXServerRenderer, { ast }));

    expect(html).not.toContain('href="javascript:alert');
  });

  it('should ignore dangerous HTML event handlers', () => {
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    const maliciousHtml = '<img src="x" onerror="alert(1)" />';
    const ast = parseMdxSync(maliciousHtml);
    const html = renderToStaticMarkup(React.createElement(MDXServerRenderer, { ast }));

    expect(html).not.toContain('onerror=');
    expect(html).not.toContain('alert(1)');

    consoleErrorSpy.mockRestore();
  });
});