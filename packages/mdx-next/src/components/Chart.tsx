"use client";

import dynamic from 'next/dynamic';
import { ParameterConfig, ChartProps } from '../interface/Chart';
import { ChartSkeleton } from './ChartSkeleton';
import { useEffect, useRef, useState } from 'react';

const DynamicalChart = dynamic(() => import('./DynamicalChart'), {
  loading: () => <ChartSkeleton />,
  ssr: false
});

const StaticChart = dynamic(() => import('./StaticChart'), {
  loading: () => <ChartSkeleton />,
  ssr: false
});

function isParameterConfig(param: unknown): param is ParameterConfig {
  return (
    typeof param === 'object' &&
    param !== null &&
    !Array.isArray(param) &&
    'min' in param
  );
}

const Chart = (props: ChartProps) => {
  const { parameter } = props;
  const ref = useRef<HTMLDivElement>(null);
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    const observer = new IntersectionObserver(([entry]) => {
      if (entry.isIntersecting) {
        setIsVisible(true);
        observer.disconnect();
      }
    }, { rootMargin: '200px' });

    if (ref.current) observer.observe(ref.current);
    return () => observer.disconnect();
  }, []);

  return (
    <div ref={ref} className='min-h-400'>
      {
        isVisible ? (
          isParameterConfig(parameter) ?
            <DynamicalChart 
              {...props} 
              parameter={parameter} 
            /> :
            <StaticChart
              caption={props.caption}
              xAxis={props.xAxis}
              yAxis={props.yAxis}
              curves={props.curves}
              resolution={props.resolution}
            /> ) :
          <ChartSkeleton/>
      }
    </div>
  )
}

export { Chart };