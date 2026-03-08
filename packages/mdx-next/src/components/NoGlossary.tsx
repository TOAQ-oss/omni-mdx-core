import React from 'react';

const NoGlossary = ({ children }: { children: React.ReactNode }) => {
  return <>{children}</>;
};

NoGlossary.isNoGlossary = true; 
NoGlossary.displayName = "NoGlossary";

export {
  NoGlossary
}