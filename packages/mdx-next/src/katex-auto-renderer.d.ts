declare module "katex/contrib/auto-render" {
  import katex from "katex";

  interface Delimiter {
    left: string;
    right: string;
    display: boolean;
  }

  interface RenderMathInElementOptions {
    delimiters?: Delimiter[];
    ignoredTags?: string[];
    ignoredClasses?: string[];
    errorCallback?: (message: string, error: Error) => void;
    macros?: Record<string, string>;
    fleqn?: boolean;
    throwOnError?: boolean;
  }

  function renderMathInElement(
    element: HTMLElement,
    options?: RenderMathInElementOptions
  ): void;

  export default renderMathInElement;
}