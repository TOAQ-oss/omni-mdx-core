"use client";

import { useMemo } from 'react';
import {
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend
} from 'recharts';
import SmartText from './SmartText';
import { ChartPoint, ChartProps } from '../interface/Chart';
import styles from '../styles/StaticChart.module.css';

const safeEval = (fn: any) => {
  if (typeof fn === 'function') return fn;
  if (typeof fn === 'string') {
    try {
      return new Function("return " + fn)();
    } catch {
      return () => 0;
    }
  }
  return () => 0;
};

export default function StaticChart({
  caption,
  xAxis,
  yAxis,
  curves,
  resolution = 100
}: ChartProps) {
  const data = useMemo(() => {
    const points = [];
    const [minX, maxX] = xAxis.domain;
    const step = (maxX - minX) / resolution;
    const parsedCurves = curves.map(c => ({
        ...c,
        realFn: safeEval(c.fn) 
    }));

    for (let x = minX; x <= maxX; x += step) {
      const point: ChartPoint = { x: Number(x.toFixed(2)) };
      parsedCurves.forEach(curve => {
        point[curve.id] = curve.realFn(x); 
      });
      points.push(point);
    }
    return points;
  }, [xAxis.domain, curves, resolution]);

  return (
    <div className={`${styles.chartWrapper} counter-figure`}>
      <div className={styles.content}>
        <div className={styles.chartArea}>
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={data} margin={{ top: 10, right: 30, left: 20, bottom: 20 }}>
              <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="#f0f0f0" />
              
              <XAxis 
                dataKey="x" 
                type="number"
                domain={xAxis.domain} 
                tickCount={xAxis.tickCount || 10}
                tick={{ 
                  fontSize: 12, 
                  fill: '#6b7280',
                  pointerEvents: 'none'
                }}
                axisLine={{ stroke: '#E5E7EB' }}
                tickLine={false}
                label={{ 
                  value: xAxis.label, 
                  position: 'bottom', 
                  offset: 0, 
                  fill: '#6b7280',
                  fontSize: 13, 
                  pointerEvents: 'none' 
                }}
              />

              <YAxis 
                domain={yAxis.domain} 
                tick={{ 
                  fontSize: 12, 
                  fill: '#6b7280',
                  pointerEvents: 'none'
                }}
                axisLine={false}
                tickLine={false}
                label={{ 
                  value: yAxis.label, 
                  angle: -90, 
                  position: 'center', 
                  fill: '#6b7280',
                  fontSize: 13, 
                  pointerEvents: 'none',
                  dx: -10
                }}
              />

              <Tooltip 
                contentStyle={{ borderRadius: '8px', border: 'none', boxShadow: '0 4px 12px rgba(0,0,0,0.08)', backgroundColor: '#37373a' }}
                labelStyle={{ color: '#9CA3AF', marginBottom: '4px' }}
                formatter={(value: number | string | undefined) => typeof value === 'number' ? value.toFixed(3) : value}
              />
              
              <Legend verticalAlign="top" height={36} iconType="circle" />

              {curves.map((curve) => (
                <Line
                  key={curve.id}
                  type="monotone"
                  dataKey={curve.id}
                  name={curve.label}
                  stroke={curve.color}
                  strokeWidth={curve.strokeWidth || 2}
                  strokeDasharray={curve.strokeDasharray}
                  dot={false}
                  activeDot={{ r: 6 }}
                  isAnimationActive={false} 
                />
              ))}
            </LineChart>
          </ResponsiveContainer>
        </div>
        {caption && (
          <div className={styles.captionContainer}>
            <p className={`${styles.captionText} caption-text`}>
              <SmartText>{caption}</SmartText>
            </p>
          </div>
        )}
      </div>
    </div>
  );
}