interface CurveConfig {
  id: string;
  label: string;
  color: string;
  strokeWidth?: number;
  strokeDasharray?: string;
  fn: (x: number, p?: number) => number; 
}

interface AxisConfig {
  label: string;
  domain: [number, number];
  tickCount?: number;
}

interface ParameterConfig {
  label: string | React.ReactNode;
  min: number;
  max: number;
  step?: number;
  defaultValue: number;
}

interface ChartProps {
  caption?: string | React.ReactNode;
  xAxis: AxisConfig;
  yAxis: AxisConfig;
  parameter?: ParameterConfig | []; 
  curves: CurveConfig[];
  resolution?: number;
}

interface ChartPoint {
    x: number;
    [key: string]: number;
}

export type {
    CurveConfig,
    AxisConfig,
    ParameterConfig,
    ChartProps,
    ChartPoint
}