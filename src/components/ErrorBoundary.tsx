import { Component } from 'react';
import type { ReactNode, ErrorInfo } from 'react';

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('StickyNotes Error:', error, info.componentStack);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div
          style={{
            padding: 16,
            fontFamily: "'DM Sans', sans-serif",
            fontSize: 13,
            color: '#6B2040',
            background: '#FFD6E0',
            borderRadius: 3,
            width: '100%',
            height: '100%',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            gap: 8,
          }}
        >
          <div style={{ fontSize: 24 }}>⚠️</div>
          <div style={{ fontWeight: 600 }}>Something went wrong</div>
          <div style={{ opacity: 0.6, fontSize: 11, textAlign: 'center' }}>
            {this.state.error?.message ?? 'Unknown error'}
          </div>
          <button
            onClick={() => this.setState({ hasError: false, error: null })}
            style={{
              marginTop: 8,
              padding: '4px 12px',
              border: 'none',
              borderRadius: 4,
              background: 'rgba(0,0,0,0.1)',
              cursor: 'pointer',
              fontSize: 12,
              color: 'inherit',
            }}
          >
            Retry
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
