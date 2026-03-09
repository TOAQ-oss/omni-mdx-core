"use client";
import { ReactNode } from 'react';

interface SpeakerProps {
  name: string;
  time?: string;
  children: ReactNode;
}

export const Speaker = ({ name, time, children }: SpeakerProps) => {
  return (
    <div style={{ display: 'flex', gap: '1rem', marginBottom: '1rem', padding: '0.5rem', backgroundColor: '#f9fafb', borderRadius: '0.5rem' }}>
      <div style={{ minWidth: '100px', flexShrink: 0 }}>
        <div style={{ fontWeight: 'bold', color: '#111827' }}>{name}</div>
        {time && <div style={{ fontSize: '0.75rem', color: '#6b7280' }}>⏱ {time}</div>}
      </div>
      <div style={{ flexGrow: 1, color: '#374151', fontStyle: 'italic' }}>
        &ldquo;{children}&rdquo;
      </div>
    </div>
  );
};