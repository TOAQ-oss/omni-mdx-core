import React from 'react';

interface AlertProps {
  children: React.ReactNode;
  type?: 'success' | 'error' | 'warning' | 'info' | string;
}

export function Alert({ children, type }: AlertProps) {
  const colors = type === "success" 
    ? "bg-blue-500/10 border-blue-500/20 text-blue-300" 
    : "bg-neutral-800 border-neutral-700 text-white";
    
  return (
    <div className={`p-4 border rounded-xl ${colors}`}>
      {children}
    </div>
  );
}