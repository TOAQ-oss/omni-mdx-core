"use client";
import { ReactNode } from 'react';

interface NoteProps {
  type?: 'info' | 'warning' | 'error';
  children: ReactNode;
}

export const Note = ({ type = 'info', children }: NoteProps) => {
  const styles = {
    info: { bg: '#eff6ff', border: '#3b82f6', text: '#1e3a8a' },
    warning: { bg: '#fffbeb', border: '#f59e0b', text: '#92400e' },
    error: { bg: '#fef2f2', border: '#ef4444', text: '#b91c1c' },
  };

  const currentStyle = styles[type] || styles.info;

  return (
    <div style={{
      backgroundColor: currentStyle.bg,
      borderLeft: `4px solid ${currentStyle.border}`,
      color: currentStyle.text,
      padding: '1rem',
      margin: '1.5rem 0',
      borderRadius: '0 0.5rem 0.5rem 0'
    }}>
      <strong>{type.toUpperCase()} : </strong>
      {children}
    </div>
  );
};