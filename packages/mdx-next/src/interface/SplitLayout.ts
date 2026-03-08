type SplitRatio = '50-50' | '60-40' | '40-60' | '70-30' | '30-70';
type VerticalAlign = 'top' | 'center' | 'bottom';

interface SplitLayoutProps {
  left: React.ReactNode;
  right: React.ReactNode;
  ratio?: SplitRatio;
  align?: VerticalAlign;
  reverse?: boolean;
  stickyRight?: boolean;
}

export type {
    SplitLayoutProps,
    SplitRatio,
    VerticalAlign
}