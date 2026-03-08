"use client";

import { Component, ReactNode } from 'react';

interface Props {
  children: ReactNode;
  componentName?: string;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class MDXErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="p-4 my-4 border-2 border-red-500 bg-red-50 rounded-lg">
          <h3 className="text-red-700 font-bold text-sm mb-1">
            Erreur de rendu : {this.props.componentName || 'Composant inconnu'}
          </h3>
          <p className="text-red-600 text-xs font-mono break-words">
            {this.state.error?.message}
          </p>
        </div>
      );
    }

    return this.props.children;
  }
}