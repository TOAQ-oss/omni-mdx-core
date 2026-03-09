export interface MDXConfig {
  /** 
   * A mapping of MDX tag names (e.g., 'Chart', 'h1') to React components.
   * These components will replace the standard HTML output.
   */
  components?: Record<string, React.ComponentType<any>>;
  
  /** 
   * Granular feature flags to enable/disable specific rendering capabilities.
   */
  features?: {
    /** Enables LaTeX math rendering (InlineMath and BlockMath). Defaults to true. */
    math?: boolean;
  };
}