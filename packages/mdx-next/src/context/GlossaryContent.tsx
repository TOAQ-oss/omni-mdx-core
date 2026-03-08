"use client";

import React, { createContext, useContext, useState } from 'react';

interface GlossaryItem {
  term: string;
  def: string;
}

interface GlossaryContextType {
  terms: GlossaryItem[];
  registerTerms: (newTerms: GlossaryItem[]) => void;
}

const GlossaryContext = createContext<GlossaryContextType>({ terms: [], registerTerms: () => {} });

export const GlossaryProvider = ({ children }: { children: React.ReactNode }) => {
  const [terms, setTerms] = useState<GlossaryItem[]>([]);

  const registerTerms = (newTerms: GlossaryItem[]) => {
    setTerms((prev) => {
      const existingTerms = new Set(prev.map(t => t.term));
      const uniqueNewTerms = newTerms.filter(t => !existingTerms.has(t.term));
      if (uniqueNewTerms.length === 0) return prev;
      return [...prev, ...uniqueNewTerms];
    });
  };

  return (
    <GlossaryContext.Provider value={{ terms, registerTerms }}>
      {children}
    </GlossaryContext.Provider>
  );
};

export const useGlossary = () => useContext(GlossaryContext);