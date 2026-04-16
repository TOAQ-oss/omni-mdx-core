// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { act } from 'react';
import { createRoot, Root } from 'react-dom/client';
import { MDXErrorBoundary } from '../../../src/MDXErrorBoundary';

declare global {
  var IS_REACT_ACT_ENVIRONMENT: boolean;
}

globalThis.IS_REACT_ACT_ENVIRONMENT = true;

const Bomb = ({ shouldThrow }: { shouldThrow?: boolean }) => {
  if (shouldThrow) {
    throw new Error('Defective component');
  }
  return <div>Contenu sécurisé</div>;
};

describe('MDXErrorBoundary', () => {
  let consoleErrorSpy: any;
  let container: HTMLDivElement;
  let root: Root;

  beforeEach(() => {
    consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    container = document.createElement('div');
    document.body.appendChild(container);
    root = createRoot(container);
  });

  afterEach(() => {
    consoleErrorSpy.mockRestore();
    act(() => {
      root.unmount();
    });
    container.remove();
  });

  it('renders standard children when no error occurs', () => {
    act(() => {
      root.render(
        <MDXErrorBoundary componentName="TestComponent">
          <Bomb />
        </MDXErrorBoundary>
      );
    });

    expect(container.textContent).toContain('Contenu sécurisé');
    expect(container.textContent).not.toContain('Render Error');
  });

  it('catches errors and renders the fallback UI', () => {
    act(() => {
      root.render(
        <MDXErrorBoundary componentName="MyChart">
          <Bomb shouldThrow />
        </MDXErrorBoundary>
      );
    });

    expect(container.textContent).toContain('Render Error : MyChart');
    expect(container.textContent).toContain('Defective component');

    expect(consoleErrorSpy).toHaveBeenCalledWith(
      expect.stringContaining('[MDXErrorBoundary] Error caught in component <MyChart>:'),
      expect.any(Error),
      expect.any(String)
    );
  });

  it('uses "Unknown Component" as a fallback name if none is provided', () => {
    act(() => {
      root.render(
        <MDXErrorBoundary>
          <Bomb shouldThrow />
        </MDXErrorBoundary>
      );
    });

    expect(container.textContent).toContain('Render Error : Unknown Component');
  });
});