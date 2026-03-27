import React from 'react';
import { Zap } from 'lucide-react';

export function Benchmark() {
  return (
    <div className="font-mono text-xs text-pink-400 bg-pink-400/10 p-2 rounded inline-flex items-center gap-2">
      <Zap size={12} /> Rendered in 0.04ms
    </div>
  );
}