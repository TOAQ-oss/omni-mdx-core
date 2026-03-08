"use client";

import dynamic from 'next/dynamic';

export const Chart = dynamic(() => import('./components/Chart').then(mod => mod.Chart), {
  ssr: false,
  loading: () => <div className="w-full h-[400px] bg-gray-100 rounded-xl animate-pulse" />
});