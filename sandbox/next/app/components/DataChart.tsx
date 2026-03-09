"use client";
import { parseProps } from '@toaq-oss/mdx-engine';

interface DataChartProps {
  title: string;
  data: unknown; 
}

export const DataChart = ({ title, data }: DataChartProps) => {
  const parsedData = Array.isArray(data) ? data : parseProps(data);
  const safeData = Array.isArray(parsedData) ? parsedData : [];

  const max = Math.max(...safeData, 1);

  return (
    <div style={{ border: '1px solid #e5e7eb', padding: '1rem', borderRadius: '0.5rem', margin: '1rem 0' }}>
      <h4 style={{ margin: '0 0 1rem 0', textAlign: 'center' }}>{title}</h4>
      <div style={{ display: 'flex', gap: '8px', alignItems: 'flex-end', height: '100px' }}>
        {safeData.map((val: number, i: number) => (
          <div key={i} style={{
            flex: 1,
            backgroundColor: '#3b82f6',
            height: `${(val / max) * 100}%`,
            borderRadius: '4px 4px 0 0',
            transition: 'height 0.3s ease'
          }} title={`Valeur: ${val}`} />
        ))}
      </div>
    </div>
  );
};