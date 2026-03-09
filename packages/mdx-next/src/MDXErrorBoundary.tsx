"use client";
import { Component, ErrorInfo, ReactNode } from 'react';

interface Props {
  children: ReactNode;
  /** The name of the MDX component being rendered (e.g., 'Chart', 'SplitLayout') */
  componentName?: string;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

/**
 * A dedicated Error Boundary for MDX rendering.
 *
 * If a React component injected via MDX crashes (e.g., due to a data parsing error 
 * inside a <Chart />), this boundary intercepts the error. This prevents the 
 * entire React tree from unmounting and displays a clean fallback UI to 
 * isolate the defective component.
 */
export class MDXErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  /**
   * Updates the state when an error occurs to trigger the fallback UI rendering.
   */
  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  /**
   * Intercepts the error and its contextual information.
   * This is the ideal place to hook into monitoring tools (like Sentry or Datadog) 
   * for production environments.
   */
  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error(
      `[MDXErrorBoundary] Error caught in component <${this.props.componentName || 'Unknown'}>:\n`,
      error,
      errorInfo.componentStack
    );
  }
  
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