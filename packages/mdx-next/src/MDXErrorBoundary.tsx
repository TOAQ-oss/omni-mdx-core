"use client";
import { Component, ErrorInfo, ReactNode } from 'react';

/**
 * Properties for the MDXErrorBoundary component.
 */
interface Props {
  /** The child elements to be rendered inside the boundary. */
  children: ReactNode;
  /** * The name of the MDX component being rendered (e.g., 'Chart', 'SplitLayout').
   * Used to display helpful debugging information if the component crashes. 
   */
  componentName?: string;
}

/**
 * Internal state for the MDXErrorBoundary component.
 */
interface State {
  /** Flag indicating whether an error has been caught. */
  hasError: boolean;
  /** The actual Error object that was caught, if any. */
  error: Error | null;
}

/**
 * A dedicated React Error Boundary for MDX rendering.
 *
 * If a custom React component injected via MDX crashes (e.g., due to a data parsing error 
 * inside a `<Chart />`), this boundary intercepts the error. This prevents the 
 * entire React tree from unmounting and instead displays a clean fallback UI to 
 * isolate the defective component.
 *
 * @example
 * ```tsx
 *  <MDXErrorBoundary componentName="MyCustomChart">
 *    <MyCustomChart data={badData} />
 *  </MDXErrorBoundary>
 * ```
 */
export class MDXErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  /**
   * Lifecycle method invoked after an error has been thrown by a descendant component.
   * Updates the state to trigger the fallback UI rendering.
   *
   * @param error - The error that was thrown.
   * @returns The new state object indicating an error has occurred.
   */
  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  /**
   * Lifecycle method invoked after an error has been thrown by a descendant component.
   * Intercepts the error and its contextual information.
   * This is the ideal place to hook into monitoring tools (like Sentry or Datadog) 
   * for production environments.
   *
   * @param error - The error that was thrown.
   * @param errorInfo - An object containing a `componentStack` trace indicating exactly where the error was thrown.
   */
  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error(
      `[MDXErrorBoundary] Error caught in component <${this.props.componentName || 'Unknown'}>:\n`,
      error,
      errorInfo.componentStack
    );
  }
  
  /**
   * Renders either the children components or the isolated fallback error UI if a crash occurred.
   *
   * @returns The rendered React node.
   */
  render() {
    if (this.state.hasError) {
      // Fallback UI: A clean, isolated container that doesn't break the main layout
      return (
        <div style={{ padding: '1rem', border: '2px solid #ef4444', backgroundColor: '#fef2f2', borderRadius: '0.5rem', margin: '1rem 0' }}>
          <h3 style={{ color: '#b91c1c', fontWeight: 'bold', margin: 0 }}>
            Render Error : {this.props.componentName || 'Unknown Component'}
          </h3>
          <p style={{ color: '#dc2626', fontFamily: 'monospace', fontSize: '0.875rem' }}>
            {this.state.error?.message}
          </p>
        </div>
      );
    }
    return this.props.children;
  }
}